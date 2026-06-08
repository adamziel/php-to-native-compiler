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
    for instruction in &module.instructions {
        match instruction {
            Instruction::Echo(value) => {
                out.push_str("    ptn_echo(");
                out.push_str(&emit_value(value));
                out.push_str(");\n");
            }
        }
    }
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
