use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_STATIC_MEMBER_REJECTION: &str = "LLVM static-member lowering rejects ::class constants, class constants, static property reads/writes, and dynamic static-property receivers until native class constant tables, static property storage, class context and late-static-binding resolution, visibility checks, autoload/class lookup, references/copy-on-write, and exact native static-member errors exist; phpc run handles current bounded static-member behavior";

#[test]
fn pdo_metadata_is_visible_but_connections_are_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
echo extension_loaded("pdo") ? "pdo" : "missing";
echo "|";
echo extension_loaded("pdo_mysql") ? "pdo-mysql" : "missing";
echo "|";
echo class_exists("PDO") ? "class" : "missing";
echo "|";
echo class_exists("PDOStatement") ? "statement" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "pdo|pdo-mysql|class|statement");
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
new PDO("mysql:host=localhost;dbname=wordpress", "user", "password");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object instantiation for PDO: PDO connections, drivers, statements, and host database state are not implemented in the current subset"
    );
}

#[test]
fn pdo_core_constants_are_visible_without_connection_support() {
    let execution = run_source(
        r#"<?php
echo defined("PDO::ATTR_ERRMODE") ? "defined" : "missing";
echo "|";
echo PDO::ATTR_ERRMODE;
echo ":";
echo PDO::ERRMODE_EXCEPTION;
echo ":";
echo PDO::FETCH_ASSOC;
echo ":";
echo PDO::FETCH_NUM;
echo ":";
echo PDO::FETCH_BOTH;
echo ":";
echo PDO::MYSQL_ATTR_INIT_COMMAND;
echo "|";
echo constant("PDO::ATTR_DEFAULT_FETCH_MODE");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "defined|3:2:2:3:4:1002|19");
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
echo PDO::ATTR_TIMEOUT;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 9);
    assert_eq!(error.message, "undefined constant PDO::ATTR_TIMEOUT");
}

#[test]
fn emit_ir_folds_pdo_metadata_but_rejects_pdo_instantiation() {
    let ir = emit_ir_source(
        r#"<?php
echo extension_loaded("pdo") ? "1" : "0";
echo extension_loaded("pdo_mysql") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");

    let error = emit_ir_source(
        r#"<?php
echo class_exists("PDO") ? "1" : "0";
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);

    let error = emit_ir_source(
        r#"<?php
new PDO("mysql:host=localhost;dbname=wordpress", "user", "password");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);

    let error = emit_ir_source(
        r#"<?php
echo PDO::ATTR_ERRMODE;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 9);
    assert_eq!(error.message, LLVM_STATIC_MEMBER_REJECTION);
}
