use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{BinaryOp, CastKind, IncDecOp, Instruction, Module, UnaryOp, ValueExpr};

pub fn emit_c(module: &Module) -> String {
    let mut out = String::new();
    out.push_str(RUNTIME_C);
    out.push_str("\nint main(void) {\n");
    out.push_str("    PtnRuntime runtime;\n");
    out.push_str("    ptn_runtime_init(&runtime);\n");
    let mut values = ValueEmitter::new();
    for instruction in &module.instructions {
        emit_instruction(&mut out, &mut values, instruction);
    }
    out.push_str("    ptn_runtime_free(&runtime);\n");
    out.push_str("    return 0;\n}\n");
    out
}

fn emit_instruction(out: &mut String, values: &mut ValueEmitter, instruction: &Instruction) {
    match instruction {
        Instruction::Store { name, value } => {
            let emitted_value = values.emit_value(out, value);
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&emitted_value);
            out.push_str(");\n");
        }
        Instruction::Echo(value) => {
            let emitted_value = values.emit_value(out, value);
            out.push_str("    ptn_echo(");
            out.push_str(&emitted_value);
            out.push_str(");\n");
        }
        Instruction::Increment { name, op } => {
            let current_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&current_temp);
            out.push_str(" = ptn_runtime_read_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\");\n");
            let result_temp = values.next_temp();
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ");
            out.push_str(match op {
                IncDecOp::Increment => "ptn_increment",
                IncDecOp::Decrement => "ptn_decrement",
            });
            out.push('(');
            out.push_str(&current_temp);
            out.push_str(");\n");
            out.push_str("    ptn_runtime_write_variable(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", ");
            out.push_str(&result_temp);
            out.push_str(");\n");
        }
        Instruction::InternalCall { name, arguments } => {
            let result_temp = values.emit_internal_call(out, name, arguments);
            out.push_str("    (void)");
            out.push_str(&result_temp);
            out.push_str(";\n");
        }
        Instruction::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("    if (ptn_is_truthy(");
            out.push_str(&condition_temp);
            out.push_str(")) {\n");
            for body_instruction in then_body {
                emit_instruction(out, values, body_instruction);
            }
            if !else_body.is_empty() {
                out.push_str("    } else {\n");
                for body_instruction in else_body {
                    emit_instruction(out, values, body_instruction);
                }
            }
            out.push_str("    }\n");
        }
        Instruction::While { condition, body } => {
            out.push_str("    while (1) {\n");
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("        if (!ptn_is_truthy(");
            out.push_str(&condition_temp);
            out.push_str(")) {\n");
            out.push_str("            break;\n");
            out.push_str("        }\n");
            for body_instruction in body {
                emit_instruction(out, values, body_instruction);
            }
            out.push_str("    }\n");
        }
        Instruction::DoWhile { body, condition } => {
            out.push_str("    while (1) {\n");
            for body_instruction in body {
                emit_instruction(out, values, body_instruction);
            }
            let condition_temp = values.emit_materialized_value(out, condition);
            out.push_str("        if (!ptn_is_truthy(");
            out.push_str(&condition_temp);
            out.push_str(")) {\n");
            out.push_str("            break;\n");
            out.push_str("        }\n");
            out.push_str("    }\n");
        }
    }
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
            ValueExpr::Binary { op, left, right } => self.emit_binary(out, *op, left, right),
            ValueExpr::Unary { op, expr } => {
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(match op {
                    UnaryOp::Positive => "ptn_positive",
                    UnaryOp::Negate => "ptn_negate",
                    UnaryOp::Not => "ptn_not",
                });
                out.push('(');
                out.push_str(&expr_temp);
                out.push_str(");\n");
                result_temp
            }
            ValueExpr::Cast { kind, expr } => {
                let expr_temp = self.emit_materialized_value(out, expr);
                let result_temp = self.next_temp();
                out.push_str("    PtnValue ");
                out.push_str(&result_temp);
                out.push_str(" = ");
                out.push_str(match kind {
                    CastKind::Int => "ptn_cast_int",
                    CastKind::Float => "ptn_cast_float",
                    CastKind::String => "ptn_cast_string",
                    CastKind::Bool => "ptn_cast_bool",
                });
                out.push('(');
                out.push_str(&expr_temp);
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
            ValueExpr::InternalCall { name, arguments } => {
                self.emit_internal_call(out, name, arguments)
            }
        }
    }

    fn emit_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::Concat
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr => self.emit_runtime_binary(out, op, left, right),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Identical
            | BinaryOp::NotIdentical
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => self.emit_comparison(out, op, left, right),
            BinaryOp::And | BinaryOp::Or => self.emit_short_circuit(out, op, left, right),
        }
    }

    fn emit_runtime_binary(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ");
        out.push_str(match op {
            BinaryOp::Add => "ptn_add",
            BinaryOp::Subtract => "ptn_subtract",
            BinaryOp::Multiply => "ptn_multiply",
            BinaryOp::Divide => "ptn_divide",
            BinaryOp::Modulo => "ptn_modulo",
            BinaryOp::Concat => "ptn_concat",
            BinaryOp::BitwiseAnd => "ptn_bitwise_and",
            BinaryOp::BitwiseOr => "ptn_bitwise_or",
            _ => unreachable!(),
        });
        out.push('(');
        out.push_str(&left_temp);
        out.push_str(", ");
        out.push_str(&right_temp);
        out.push_str(");\n");
        result_temp
    }

    fn emit_comparison(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let right_temp = self.emit_materialized_value(out, right);
        let result_temp = self.next_temp();
        let comparison = match op {
            BinaryOp::Equal => format!("ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::NotEqual => format!("!ptn_compare_equal({left_temp}, {right_temp})"),
            BinaryOp::Identical => format!("ptn_compare_identical({left_temp}, {right_temp})"),
            BinaryOp::NotIdentical => {
                format!("!ptn_compare_identical({left_temp}, {right_temp})")
            }
            BinaryOp::Less => format!("ptn_compare_less({left_temp}, {right_temp})"),
            BinaryOp::LessEqual => format!("ptn_compare_less_equal({left_temp}, {right_temp})"),
            BinaryOp::Greater => format!("ptn_compare_greater({left_temp}, {right_temp})"),
            BinaryOp::GreaterEqual => {
                format!("ptn_compare_greater_equal({left_temp}, {right_temp})")
            }
            _ => unreachable!(),
        };
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_bool(");
        out.push_str(&comparison);
        out.push_str(");\n");
        result_temp
    }

    fn emit_short_circuit(
        &mut self,
        out: &mut String,
        op: BinaryOp,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> String {
        let left_temp = self.emit_materialized_value(out, left);
        let result_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(";\n");
        out.push_str("    if (ptn_is_truthy(");
        out.push_str(&left_temp);
        out.push_str(")) {\n");
        match op {
            BinaryOp::And => {
                let right_value = self.emit_value(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(ptn_is_truthy(");
                out.push_str(&right_value);
                out.push_str("));\n");
                out.push_str("    } else {\n");
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(0);\n");
            }
            BinaryOp::Or => {
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(1);\n");
                out.push_str("    } else {\n");
                let right_value = self.emit_value(out, right);
                out.push_str("        ");
                out.push_str(&result_temp);
                out.push_str(" = ptn_bool(ptn_is_truthy(");
                out.push_str(&right_value);
                out.push_str("));\n");
            }
            _ => unreachable!(),
        }
        out.push_str("    }\n");
        result_temp
    }

    fn emit_materialized_value(&mut self, out: &mut String, value: &ValueExpr) -> String {
        if matches!(
            value,
            ValueExpr::Binary { .. }
                | ValueExpr::InternalCall { .. }
                | ValueExpr::Unary { .. }
                | ValueExpr::Cast { .. }
        ) {
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

    fn emit_internal_call(
        &mut self,
        out: &mut String,
        name: &str,
        arguments: &[ValueExpr],
    ) -> String {
        let result_temp = self.next_temp();
        if arguments.is_empty() {
            out.push_str("    PtnValue ");
            out.push_str(&result_temp);
            out.push_str(" = ptn_call_internal(&runtime, \"");
            out.push_str(&c_string(name));
            out.push_str("\", 0, NULL);\n");
            return result_temp;
        }

        let mut temps = Vec::with_capacity(arguments.len());
        for argument in arguments {
            temps.push(self.emit_materialized_value(out, argument));
        }

        let args_temp = self.next_temp();
        out.push_str("    PtnValue ");
        out.push_str(&args_temp);
        out.push_str("[] = { ");
        out.push_str(&temps.join(", "));
        out.push_str(" };\n");
        out.push_str("    PtnValue ");
        out.push_str(&result_temp);
        out.push_str(" = ptn_call_internal(&runtime, \"");
        out.push_str(&c_string(name));
        out.push_str("\", ");
        out.push_str(&arguments.len().to_string());
        out.push_str(", ");
        out.push_str(&args_temp);
        out.push_str(");\n");
        result_temp
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

typedef PtnValue (*PtnInternalFunctionHandler)(PtnRuntime *runtime, size_t argc, const PtnValue *args);

typedef struct {
    const char *name;
    size_t min_args;
    size_t max_args;
    PtnInternalFunctionHandler handler;
} PtnInternalFunction;

#define PTN_VARIADIC_ARGS ((size_t)-1)

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

static void ptn_emit_undefined_function_error(PtnDiagnosticSink *diagnostics, const char *name) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Call to undefined function ", stream);
    fputs(name, stream);
    fputs("()\n", stream);
}

static void ptn_emit_argument_count_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t min_args,
    size_t argc
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at least ", stream);
    fprintf(stream, "%zu", min_args);
    fputs(" argument", stream);
    if (min_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
}

static void ptn_emit_too_many_arguments_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t max_args,
    size_t argc
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at most ", stream);
    fprintf(stream, "%zu", max_args);
    fputs(" argument", stream);
    if (max_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
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

static PTN_UNUSED PtnValue ptn_negate(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(-number.floating);
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(-(double)number.integer);
    }
    return ptn_int(-number.integer);
}

static PTN_UNUSED PtnValue ptn_positive(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED int ptn_is_truthy(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return value.as.boolean != 0;
        case PTN_INT:
            return value.as.integer != 0;
        case PTN_FLOAT:
            return value.as.floating != 0.0;
        case PTN_STRING:
            return value.as.string[0] != '\0' && strcmp(value.as.string, "0") != 0;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_not(PtnValue value) {
    return ptn_bool(!ptn_is_truthy(value));
}

static PTN_UNUSED PtnValue ptn_cast_int(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_int((int64_t)number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED PtnValue ptn_cast_float(PtnValue value) {
    PtnNumber number = ptn_to_number(value);
    return ptn_float(number.floating);
}

static PTN_UNUSED void ptn_abort_arithmetic_error(const char *message) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(1);
}

static PTN_UNUSED int ptn_is_number_type(PtnValue value) {
    return value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED int ptn_is_numeric_string(const char *string, double *number) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    double parsed = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    if (*end != '\0') {
        return 0;
    }
    *number = parsed;
    return 1;
}

static PTN_UNUSED int ptn_comparison_numeric_value(PtnValue value, double *number) {
    switch (value.type) {
        case PTN_INT:
            *number = (double)value.as.integer;
            return 1;
        case PTN_FLOAT:
            *number = value.as.floating;
            return 1;
        case PTN_STRING:
            return ptn_is_numeric_string(value.as.string, number);
        case PTN_NULL:
        case PTN_BOOL:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_numbers(double left, double right) {
    if (left < right) {
        return -1;
    }
    if (left > right) {
        return 1;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_strings(const char *left, const char *right) {
    int compared = strcmp(left, right);
    return compared < 0 ? -1 : (compared > 0 ? 1 : 0);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
        snprintf(buffer, buffer_len, "%lld", (long long)value.as.integer);
    } else {
        snprintf(buffer, buffer_len, "%.14g", value.as.floating);
    }
}

static PTN_UNUSED int ptn_compare_number_and_string(PtnValue number, const char *string, int number_is_left) {
    char number_string[128];
    ptn_number_value_to_string(number, number_string, sizeof(number_string));
    int compared = ptn_compare_strings(number_string, string);
    return number_is_left ? compared : -compared;
}

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right) {
    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_is_truthy(left) == ptn_is_truthy(right);
    }
    if (left.type == PTN_NULL || right.type == PTN_NULL) {
        if (left.type == PTN_NULL && right.type == PTN_NULL) {
            return 1;
        }
        PtnValue other = left.type == PTN_NULL ? right : left;
        switch (other.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return ptn_is_truthy(other) == 0;
            case PTN_INT:
                return other.as.integer == 0;
            case PTN_FLOAT:
                return other.as.floating == 0.0;
            case PTN_STRING:
                return other.as.string[0] == '\0';
        }
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number) == 0;
    }
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return strcmp(left.as.string, right.as.string) == 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right) {
    if (left.type != right.type) {
        return 0;
    }
    switch (left.type) {
        case PTN_NULL:
            return 1;
        case PTN_BOOL:
            return left.as.boolean == right.as.boolean;
        case PTN_INT:
            return left.as.integer == right.as.integer;
        case PTN_FLOAT:
            return left.as.floating == right.as.floating;
        case PTN_STRING:
            return strcmp(left.as.string, right.as.string) == 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right) {
    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
    }
    if (left.type == PTN_NULL && right.type == PTN_NULL) {
        return 0;
    }
    if (left.type == PTN_NULL) {
        if (ptn_is_number_type(right)) {
            double right_number = right.type == PTN_INT ? (double)right.as.integer : right.as.floating;
            return ptn_compare_numbers(0.0, right_number);
        }
        if (right.type == PTN_STRING) {
            return ptn_compare_strings("", right.as.string);
        }
    }
    if (right.type == PTN_NULL) {
        if (ptn_is_number_type(left)) {
            double left_number = left.type == PTN_INT ? (double)left.as.integer : left.as.floating;
            return ptn_compare_numbers(left_number, 0.0);
        }
        if (left.type == PTN_STRING) {
            return ptn_compare_strings(left.as.string, "");
        }
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_compare_strings(left.as.string, right.as.string);
    }
    if (ptn_is_number_type(left) && right.type == PTN_STRING) {
        return ptn_compare_number_and_string(left, right.as.string, 1);
    }
    if (left.type == PTN_STRING && ptn_is_number_type(right)) {
        return ptn_compare_number_and_string(right, left.as.string, 0);
    }
    return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
}

static PTN_UNUSED int ptn_compare_less(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) < 0;
}

static PTN_UNUSED int ptn_compare_less_equal(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) <= 0;
}

static PTN_UNUSED int ptn_compare_greater(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) > 0;
}

static PTN_UNUSED int ptn_compare_greater_equal(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) >= 0;
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

static PTN_UNUSED PtnValue ptn_subtract(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating - right_number.floating);
    }

    if ((right_number.integer < 0 && left_number.integer > INT64_MAX + right_number.integer) ||
        (right_number.integer > 0 && left_number.integer < INT64_MIN + right_number.integer)) {
        return ptn_float((double)left_number.integer - (double)right_number.integer);
    }
    return ptn_int(left_number.integer - right_number.integer);
}

static PTN_UNUSED int ptn_multiply_overflows(int64_t left, int64_t right) {
    if (left == 0 || right == 0) {
        return 0;
    }
    if (left > 0) {
        if (right > 0) {
            return left > INT64_MAX / right;
        }
        return right < INT64_MIN / left;
    }
    if (right > 0) {
        return left < INT64_MIN / right;
    }
    return right < INT64_MAX / left;
}

static PTN_UNUSED PtnValue ptn_multiply(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating * right_number.floating);
    }

    if (ptn_multiply_overflows(left_number.integer, right_number.integer)) {
        return ptn_float((double)left_number.integer * (double)right_number.integer);
    }
    return ptn_int(left_number.integer * right_number.integer);
}

static PTN_UNUSED int64_t ptn_number_to_integer(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return (int64_t)number.floating;
    }
    return number.integer;
}

static PTN_UNUSED PtnValue ptn_divide(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (right_number.floating == 0.0) {
        ptn_abort_arithmetic_error("Division by zero");
    }

    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer == INT64_MIN && right_number.integer == -1) {
            return ptn_float((double)left_number.integer / (double)right_number.integer);
        }
        if (left_number.integer % right_number.integer == 0) {
            return ptn_int(left_number.integer / right_number.integer);
        }
    }
    return ptn_float(left_number.floating / right_number.floating);
}

static PTN_UNUSED PtnValue ptn_modulo(PtnValue left, PtnValue right) {
    int64_t left_integer = ptn_number_to_integer(ptn_to_number(left));
    int64_t right_integer = ptn_number_to_integer(ptn_to_number(right));
    if (right_integer == 0) {
        ptn_abort_arithmetic_error("Modulo by zero");
    }
    if (left_integer == INT64_MIN && right_integer == -1) {
        return ptn_int(0);
    }
    return ptn_int(left_integer % right_integer);
}

static PTN_UNUSED PtnValue ptn_increment(PtnValue value) {
    return ptn_add(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_decrement(PtnValue value) {
    return ptn_subtract(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_bitwise_string_and(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left[i] & (unsigned char)right[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_or(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len > right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        unsigned char left_byte = i < left_len ? (unsigned char)left[i] : 0;
        unsigned char right_byte = i < right_len ? (unsigned char)right[i] : 0;
        result[i] = (char)(left_byte | right_byte);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED int64_t ptn_value_to_integer(PtnValue value) {
    return ptn_number_to_integer(ptn_to_number(value));
}

static PTN_UNUSED PtnValue ptn_bitwise_and(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_and(left.as.string, right.as.string);
    }
    return ptn_int(ptn_value_to_integer(left) & ptn_value_to_integer(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_or(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_or(left.as.string, right.as.string);
    }
    return ptn_int(ptn_value_to_integer(left) | ptn_value_to_integer(right));
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

static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value) {
    return ptn_owned_string(ptn_value_to_string(value));
}

static PTN_UNUSED PtnValue ptn_cast_bool(PtnValue value) {
    return ptn_bool(ptn_is_truthy(value));
}

static PTN_UNUSED PtnValue ptn_gettype_value(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_string("NULL");
        case PTN_BOOL:
            return ptn_string("boolean");
        case PTN_INT:
            return ptn_string("integer");
        case PTN_FLOAT:
            return ptn_string("double");
        case PTN_STRING:
            return ptn_string("string");
    }
    return ptn_string("unknown type");
}

static PTN_UNUSED PtnValue ptn_is_type(PtnValue value, PtnType type) {
    return ptn_bool(value.type == type);
}

static PTN_UNUSED PtnValue ptn_is_scalar(PtnValue value) {
    return ptn_bool(
        value.type == PTN_BOOL ||
        value.type == PTN_INT ||
        value.type == PTN_FLOAT ||
        value.type == PTN_STRING
    );
}

static PTN_UNUSED void ptn_echo(PtnValue value) {
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

static void ptn_var_dump_value(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            fputs("NULL\n", stdout);
            break;
        case PTN_BOOL:
            fputs(value.as.boolean ? "bool(true)\n" : "bool(false)\n", stdout);
            break;
        case PTN_INT:
            printf("int(%lld)\n", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            printf("float(%.14g)\n", value.as.floating);
            break;
        case PTN_STRING:
            printf("string(%zu) \"", strlen(value.as.string));
            fputs(value.as.string, stdout);
            fputs("\"\n", stdout);
            break;
    }
}

static PtnValue ptn_internal_var_dump(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    for (size_t i = 0; i < argc; i++) {
        ptn_var_dump_value(args[i]);
    }
    return ptn_null();
}

static PtnValue ptn_internal_strlen(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    char *string = ptn_value_to_string(args[0]);
    size_t len = strlen(string);
    free(string);
    return ptn_int((int64_t)len);
}

static PtnValue ptn_internal_gettype(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_gettype_value(args[0]);
}

static PtnValue ptn_internal_is_null(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_type(args[0], PTN_NULL);
}

static PtnValue ptn_internal_is_bool(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_type(args[0], PTN_BOOL);
}

static PtnValue ptn_internal_is_int(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_type(args[0], PTN_INT);
}

static PtnValue ptn_internal_is_float(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_type(args[0], PTN_FLOAT);
}

static PtnValue ptn_internal_is_string(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_type(args[0], PTN_STRING);
}

static PtnValue ptn_internal_is_scalar(PtnRuntime *runtime, size_t argc, const PtnValue *args) {
    (void)runtime;
    (void)argc;
    return ptn_is_scalar(args[0]);
}

static PTN_UNUSED PtnValue ptn_call_internal(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args) {
    static const PtnInternalFunction functions[] = {
        { "var_dump", 1, PTN_VARIADIC_ARGS, ptn_internal_var_dump },
        { "strlen", 1, 1, ptn_internal_strlen },
        { "gettype", 1, 1, ptn_internal_gettype },
        { "is_null", 1, 1, ptn_internal_is_null },
        { "is_bool", 1, 1, ptn_internal_is_bool },
        { "is_int", 1, 1, ptn_internal_is_int },
        { "is_integer", 1, 1, ptn_internal_is_int },
        { "is_long", 1, 1, ptn_internal_is_int },
        { "is_float", 1, 1, ptn_internal_is_float },
        { "is_double", 1, 1, ptn_internal_is_float },
        { "is_string", 1, 1, ptn_internal_is_string },
        { "is_scalar", 1, 1, ptn_internal_is_scalar },
    };

    for (size_t i = 0; i < sizeof(functions) / sizeof(functions[0]); i++) {
        const PtnInternalFunction *function = &functions[i];
        if (strcmp(function->name, name) == 0) {
            if (argc < function->min_args) {
                ptn_emit_argument_count_error(&runtime->diagnostics, name, function->min_args, argc);
                exit(255);
            }
            if (function->max_args != PTN_VARIADIC_ARGS && argc > function->max_args) {
                ptn_emit_too_many_arguments_error(&runtime->diagnostics, name, function->max_args, argc);
                exit(255);
            }
            return function->handler(runtime, argc, args);
        }
    }

    ptn_emit_undefined_function_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}
"#;
