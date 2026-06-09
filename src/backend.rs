use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{Instruction, Module, ValueExpr};

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    out.push_str(RUNTIME_C);
    out.push_str("\nint main(void) {\n");
    out.push_str("    PtnRuntime runtime;\n");
    out.push_str("    ptn_runtime_init(&runtime);\n");
    for instruction in &module.instructions {
        match instruction {
            Instruction::Store { name, value } => {
                out.push_str("    ptn_runtime_write_variable(&runtime, \"");
                out.push_str(&c_string(name));
                out.push_str("\", ");
                out.push_str(&emit_value(value));
                out.push_str(");\n");
            }
            Instruction::Echo(value) => {
                out.push_str("    ptn_echo(");
                out.push_str(&emit_value(value));
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

fn emit_value(value: &ValueExpr) -> String {
    match value {
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

const RUNTIME_C: &str = r#"#include <stdio.h>
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

static void ptn_abort_out_of_memory(void) {
    fputs("Fatal error: out of memory\n", stderr);
    exit(1);
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
