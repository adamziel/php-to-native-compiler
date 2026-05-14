use std::fs;

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

const LLVM_NAMESPACE_REJECTION: &str = "LLVM namespace lowering rejects namespace declarations, namespace-qualified names, namespace imports, and namespace-aware name resolution until native symbol tables, namespace context, aliases/imports, fallback function/constant lookup, class/autoload lookup, and exact native error behavior exist; phpc run handles current namespace behavior";

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn unbracketed_namespace_resolves_declared_classes_and_class_metadata_names() {
    let execution = run_source(
        r#"<?php
namespace App\Core;

class Base {}
class Service extends Base {
    public static function label() {
        return self::class;
    }
}

$service = new Service();
echo Service::class, "\n";
echo Service::label(), "\n";
echo get_class($service), "\n";
echo get_parent_class($service), "\n";
echo class_exists(Service::class) ? "yes" : "no";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "App\\Core\\Service\nApp\\Core\\Service\nApp\\Core\\Service\nApp\\Core\\Base\nyes"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn simple_class_import_aliases_resolve_static_and_new_references() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "imports"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create namespace import fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
namespace Vendor\Lib;

class Tool {
    public static function label() {
        return self::class;
    }
}
"#,
    )
    .expect("write namespaced library fixture");

    let source = r#"<?php
namespace App;

use Vendor\Lib\Tool as ImportedTool;
require 'lib.php';

$tool = new ImportedTool();
echo ImportedTool::class, "\n";
echo ImportedTool::label(), "\n";
echo get_class($tool), "\n";
echo \Vendor\Lib\Tool::class;
"#;
    fs::write(&main, source).expect("write namespace import main fixture");

    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        "Vendor\\Lib\\Tool\nVendor\\Lib\\Tool\nVendor\\Lib\\Tool\nVendor\\Lib\\Tool"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn namespace_relative_class_names_resolve_against_current_namespace() {
    let execution = run_source(
        r#"<?php
namespace App\Demo;

class Box {}
echo namespace\Box::class, "\n";
$box = new namespace\Box();
echo get_class($box);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "App\\Demo\\Box\nApp\\Demo\\Box");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unbracketed_namespace_resolves_function_declarations_and_unqualified_calls() {
    let execution = run_source(
        r#"<?php
namespace App\Core;

function label($name = "Ada") {
    return __FUNCTION__ . ":" . $name;
}

echo label(), "\n";
echo LABEL("Grace"), "\n";
echo strlen("abc"), "\n";
echo function_exists("App\\Core\\label") ? "yes" : "no", "\n";
echo function_exists("APP\\CORE\\LABEL") ? "yes" : "no", "\n";
echo function_exists("label") ? "yes" : "no";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "App\\Core\\label:Ada\nApp\\Core\\label:Grace\n3\nyes\nyes\nno"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn duplicate_namespaced_function_declarations_share_case_insensitive_key() {
    let error = run_source(
        r#"<?php
namespace App;
function label() {}
function LABEL() {}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "function App\\LABEL() is already defined");
}

#[test]
fn unsupported_namespace_forms_keep_stable_parse_boundaries() {
    let cases = [
        (
            r#"<?php
namespace App\Demo {
    echo "blocked";
}
"#,
            2,
            1,
            "unsupported namespace declaration: bracketed namespace blocks are not implemented",
        ),
        (
            r#"<?php
namespace App;
namespace Other;
"#,
            3,
            1,
            "unsupported namespace declaration: multiple namespace declarations are not implemented",
        ),
        (
            r#"<?php
if (true) {
    namespace App;
}
"#,
            3,
            5,
            "unsupported namespace declaration: namespace declarations are only implemented at file scope",
        ),
        (
            r#"<?php
namespace App;
const NAME = "app";
"#,
            3,
            1,
            "unsupported const declaration: namespace-qualified constant declarations are not implemented",
        ),
        (
            r#"<?php
use function App\Demo\make_service;
"#,
            2,
            1,
            "unsupported use declaration: only simple class imports are implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn native_lowering_rejects_namespace_context_before_scalar_folds() {
    for source in [
        "<?php\nnamespace App;\necho \"ok\";\n",
        "<?php\nnamespace App;\necho strlen(\"abc\");\n",
        "<?php\nnamespace App;\nuse Vendor\\Lib\\Tool;\necho Tool::class;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, LLVM_NAMESPACE_REJECTION);
    }
}

#[test]
fn assembly_lowering_rejects_namespace_context_before_backend_execution() {
    let error = emit_asm_source("<?php\nnamespace App;\necho \"ok\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_NAMESPACE_REJECTION);
}
