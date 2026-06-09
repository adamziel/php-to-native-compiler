use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{BinaryOp, Instruction, Module, ValueExpr};

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    out.push_str(RUNTIME_C);
    out.push_str("\nint main(void) {\n");
    out.push_str("    PtnRuntime runtime;\n");
    out.push_str("    ptn_runtime_init(&runtime);\n");
    let mut values = ValueEmitter::new();
    for instruction in &module.instructions {
        match instruction {
            Instruction::Store { name, value } => {
                let emitted_value = values.emit_value(&mut out, value);
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&emitted_value);
                out.push_str(");\n");
            }
            Instruction::Echo(value) => {
                let emitted_value = values.emit_value(&mut out, value);
                out.push_str("    ptn_echo(");
                out.push_str(&emitted_value);
                out.push_str(");\n");
            }
        }
    }
    out.push_str("    ptn_runtime_free(&runtime);\n");
    out.push_str("    return 0;\n}\n");
    out
}

pub fn compile_c(c_source: &str, output: &Path) -> Result<()> {
    let c_path = output.with_extension("c");
    fs::write(&c_path, c_source).map_err(|error| {
        Diagnostic::new(
            format!(
                "failed to write generated C source {}: {error}",
                c_path.display()
            ),
            None,
        )
    })?;
    let status = Command::new("cc")
        .arg("-std=c11")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-O2")
        .arg(&c_path)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| Diagnostic::new(format!("failed to launch cc: {error}"), None))?;
    if status.success() {
        Ok(())
    } else {
        Err(Diagnostic::new(
            format!(
                "cc failed compiling {} to {}",
                display_os(c_path.as_os_str()),
                display_os(output.as_os_str())
            ),
            None,
        ))
    }
}

struct ValueEmitter {
    next_temp: usize,
}

impl ValueEmitter {
    fn new() -> Self {
        Self { next_temp: 0 }
    }

    fn emit_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Binary { op, left, right } => {
                let left_temp = self.emit_materialized_value(out, left);
                let right_temp = self.emit_materialized_value(out, right);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(match op {
                    BinaryOp::Add => "ptn_add",
                    BinaryOp::Concat => "ptn_concat",
                });
                out.push('(');
                out.push_str(&left_temp);
                out.push_str(", ");
                out.push_str(&right_temp);
                out.push_str(");\n");
                result_temp
            }
            ValueExpr::String(value) => format!("ptn_string(\"{}\")", c_string(value)),
            ValueExpr::Int(value) => format!("ptn_int({value})"),
            ValueExpr::Float(value) => format!("ptn_float({value:?})"),
            ValueExpr::Bool(true) => "ptn_bool(1)".to_string(),
            ValueExpr::Bool(false) => "ptn_bool(0)".to_string(),
            ValueExpr::Null => "ptn_null()".to_string(),
            ValueExpr::Load(name) => format!(
                "ptn_runtime_read_variable(&runtime, \"{}\")",
                c_string(name)
            ),
        }
    }

    fn emit_materialized_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        if matches!(value, ValueExpr::Binary { .. }) {
            return self.emit_value(out, value);
        }

        let temp = self.next_temp();
        let emitted_value = self.emit_value(out, value);
        out.push_str("    PtnValue ");
        out.push_str(&temp);
        out.push_str(" = ");
        out.push_str(&emitted_value);
        out.push_str(";\n");
        temp
    }

    fn next_temp(&mut self) -> String {
        let temp = format!("ptn_tmp_{}", self.next_temp);
        self.next_temp += 1;
        temp
    }
}

fn c_string(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(byte as char),
            _ => out.push_str(&format!("\\x{byte:02x}")),
        }
    }
    out
}

fn display_os(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

const RUNTIME_C: &str = r#"#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING
} PtnType;

typedef struct {
    PtnType type;
    union {
        int boolean;
        int64_t integer;
        double floating;
        const char *string;
    } as;
} PtnValue;

typedef enum {
    PTN_NUMBER_INT,
    PTN_NUMBER_FLOAT
} PtnNumberType;

typedef struct {
    PtnNumberType type;
    int64_t integer;
    double floating;
} PtnNumber;

typedef struct {
    const char *name;
    PtnValue value;
} PtnSymbol;

typedef struct {
    PtnSymbol *items;
    size_t len;
    size_t capacity;
} PtnSymbolTable;

typedef struct {
    FILE *stream;
} PtnDiagnosticSink;

typedef struct {
    PtnSymbolTable symbols;
    PtnDiagnosticSink diagnostics;
} PtnRuntime;

static PTN_UNUSED PtnValue ptn_null(void) {
    PtnValue value;
    value.type = PTN_NULL;
    return value;
}

static PTN_UNUSED PtnValue ptn_bool(int boolean) {
    PtnValue value;
    value.type = PTN_BOOL;
    value.as.boolean = boolean ? 1 : 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_int(int64_t integer) {
    PtnValue value;
    value.type = PTN_INT;
    value.as.integer = integer;
    return value;
}

static PTN_UNUSED PtnValue ptn_float(double floating) {
    PtnValue value;
    value.type = PTN_FLOAT;
    value.as.floating = floating;
    return value;
}

static PTN_UNUSED PtnValue ptn_string(const char *string) {
    PtnValue value;
    value.type = PTN_STRING;
    value.as.string = string;
    return value;
}

static PTN_UNUSED PtnValue ptn_owned_string(char *string) {
    PtnValue value;
    value.type = PTN_STRING;
    value.as.string = string;
    return value;
}

static void ptn_abort_out_of_memory(void) {
    fputs("Fatal error: out of memory\n", stderr);
    exit(1);
}

static PTN_UNUSED char *ptn_duplicate_string(const char *string) {
    size_t len = strlen(string);
    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(copy, string, len + 1);
    return copy;
}

static void ptn_symbols_init(PtnSymbolTable *symbols) {
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
}

static void ptn_symbols_free(PtnSymbolTable *symbols) {
    free(symbols->items);
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
}

static size_t ptn_symbols_find(PtnSymbolTable *symbols, const char *name) {
    for (size_t i = 0; i < symbols->len; i++) {
        if (strcmp(symbols->items[i].name, name) == 0) {
            return i;
        }
    }
    return symbols->len;
}

static PTN_UNUSED void ptn_symbols_set(PtnSymbolTable *symbols, const char *name, PtnValue value) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        symbols->items[index].value = value;
        return;
    }
    if (symbols->len == symbols->capacity) {
        size_t new_capacity = symbols->capacity == 0 ? 8 : symbols->capacity * 2;
        PtnSymbol *new_items = realloc(symbols->items, new_capacity * sizeof(PtnSymbol));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        symbols->items = new_items;
        symbols->capacity = new_capacity;
    }
    symbols->items[symbols->len].name = name;
    symbols->items[symbols->len].value = value;
    symbols->len++;
}

static int ptn_symbols_get(PtnSymbolTable *symbols, const char *name, PtnValue *out) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        *out = symbols->items[index].value;
        return 1;
    }
    return 0;
}

static void ptn_diagnostics_init(PtnDiagnosticSink *diagnostics, FILE *stream) {
    diagnostics->stream = stream;
}

static void ptn_emit_undefined_variable_warning(PtnDiagnosticSink *diagnostics, const char *name) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Warning: Undefined variable $", stream);
    fputs(name, stream);
    fputc('\n', stream);
}

static void ptn_runtime_init(PtnRuntime *runtime) {
    ptn_symbols_init(&runtime->symbols);
    ptn_diagnostics_init(&runtime->diagnostics, stderr);
}

static void ptn_runtime_free(PtnRuntime *runtime) {
    ptn_symbols_free(&runtime->symbols);
}

static PTN_UNUSED void ptn_runtime_write_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(&runtime->symbols, name, value);
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return value;
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name);
    return ptn_null();
}

static PTN_UNUSED PtnNumber ptn_number_int(int64_t integer) {
    PtnNumber number;
    number.type = PTN_NUMBER_INT;
    number.integer = integer;
    number.floating = (double)integer;
    return number;
}

static PTN_UNUSED PtnNumber ptn_number_float(double floating) {
    PtnNumber number;
    number.type = PTN_NUMBER_FLOAT;
    number.integer = 0;
    number.floating = floating;
    return number;
}

static PTN_UNUSED int ptn_contains_float_marker(const char *start, const char *end) {
    for (const char *cursor = start; cursor < end; cursor++) {
        if (*cursor == '.' || *cursor == 'e' || *cursor == 'E') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED PtnNumber ptn_string_to_number(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return ptn_number_int(0);
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return ptn_number_int(0);
    }

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        return ptn_number_int((int64_t)integer);
    }
    return ptn_number_float(floating);
}

static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_number_int(0);
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING:
            return ptn_string_to_number(value.as.string);
    }
    return ptn_number_int(0);
}

static PTN_UNUSED PtnValue ptn_add(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating + right_number.floating);
    }

    if ((right_number.integer > 0 && left_number.integer > INT64_MAX - right_number.integer) ||
        (right_number.integer < 0 && left_number.integer < INT64_MIN - right_number.integer)) {
        return ptn_float((double)left_number.integer + (double)right_number.integer);
    }
    return ptn_int(left_number.integer + right_number.integer);
}

static PTN_UNUSED char *ptn_value_to_string(PtnValue value) {
    char buffer[128];
    int written = 0;

    switch (value.type) {
        case PTN_NULL:
            return ptn_duplicate_string("");
        case PTN_BOOL:
            return ptn_duplicate_string(value.as.boolean ? "1" : "");
        case PTN_INT:
            written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            written = snprintf(buffer, sizeof(buffer), "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            return ptn_duplicate_string(value.as.string);
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_duplicate_string(buffer);
}

static PTN_UNUSED PtnValue ptn_concat(PtnValue left, PtnValue right) {
    char *left_string = ptn_value_to_string(left);
    char *right_string = ptn_value_to_string(right);
    size_t left_len = strlen(left_string);
    size_t right_len = strlen(right_string);
    if (left_len > SIZE_MAX - right_len) {
        ptn_abort_out_of_memory();
    }
    size_t joined_len = left_len + right_len;
    if (joined_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *joined = malloc(joined_len + 1);
    if (joined == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(joined, left_string, left_len);
    memcpy(joined + left_len, right_string, right_len + 1);
    free(left_string);
    free(right_string);
    return ptn_owned_string(joined);
}

static void ptn_echo(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                fputs("1", stdout);
            }
            break;
        case PTN_INT:
            printf("%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            printf("%.14g", value.as.floating);
            break;
        case PTN_STRING:
            fputs(value.as.string, stdout);
            break;
    }
}
"#;
