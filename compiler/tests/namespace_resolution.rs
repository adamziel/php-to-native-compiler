use std::fs;

use php_compiler::error::Phase;
use php_compiler::{
    codegen::emit_native_executable_c_source, emit_asm_source, emit_ir_source, parse, run_source,
    run_source_with_source_file,
};

const LLVM_NAMESPACE_REJECTION: &str = "LLVM namespace lowering rejects namespace declarations, namespace-qualified names, namespace imports, and namespace-aware name resolution until native symbol tables, namespace context, aliases/imports, fallback function/constant lookup, class/autoload lookup, and exact native error behavior exist; phpc run handles current namespace behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls outside the bounded generated-C user-function frame subset, including unknown user functions, callable builtins outside define()/constant()/defined(), arity-mismatched direct calls, unsupported by-reference argument binding, and unsupported dynamic string-valued calls, until full callable lookup, full arity/type diagnostics, callbacks, and cleanup handoff exist; generated-native C lowers supported by-value fixed/default/variadic direct, supported direct and compiler-known single-target by-reference frames, finite known-string dynamic, and runtime string-valued dynamic user-function frames";
const ASSEMBLY_GLOBAL_CONSTANT_REJECTION: &str = "assembly global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";

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
fn leading_global_function_calls_bypass_namespace_function_fallback() {
    let execution = run_source(
        r#"<?php
namespace App\Core;

function strlen($value) {
    return 99;
}

function label($value) {
    return "local:" . $value;
}

echo strlen("abc"), "\n";
echo \strlen("abc"), "\n";
echo \App\Core\label("name");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "99\n3\nlocal:name");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_imports_resolve_aliases_and_keep_non_imported_fallback() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "function-imports"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create function import fixture directory");
    let main = root.join("index.php");
    let lib = root.join("functions.php");

    fs::write(
        &lib,
        r#"<?php
namespace Vendor\Tools;

function label($value) {
    return __FUNCTION__ . ":" . $value;
}

function other($value) {
    return __FUNCTION__ . ":" . $value;
}
"#,
    )
    .expect("write function import library fixture");

    let source = r#"<?php
namespace App\Demo;

use function Vendor\Tools\label as vendor_label, Vendor\Tools\other;
require 'functions.php';

function label($value) {
    return __FUNCTION__ . ":" . $value;
}

echo vendor_label("a"), "\n";
echo other("b"), "\n";
echo label("c"), "\n";
echo strlen("abc");
"#;
    fs::write(&main, source).expect("write function import main fixture");

    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        "Vendor\\Tools\\label:a\nVendor\\Tools\\other:b\nApp\\Demo\\label:c\n3"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(lib);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(root);
}

#[test]
fn qualified_function_calls_resolve_relative_namespace_and_exact_global_names() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "qualified-function-calls"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create qualified function fixture directory");
    let main = root.join("index.php");
    let lib = root.join("functions.php");

    fs::write(
        &lib,
        r#"<?php
namespace App\Demo\Sub;

function helper($value) {
    return __FUNCTION__ . ":" . $value;
}
"#,
    )
    .expect("write qualified function library fixture");

    let source = r#"<?php
namespace App\Demo;
require 'functions.php';

function helper($value) {
    return __FUNCTION__ . ":" . $value;
}

echo namespace\helper("namespace"), "\n";
echo Sub\helper("relative"), "\n";
echo \App\Demo\Sub\helper("exact");
"#;
    fs::write(&main, source).expect("write qualified function main fixture");

    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        "App\\Demo\\helper:namespace\nApp\\Demo\\Sub\\helper:relative\nApp\\Demo\\Sub\\helper:exact"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(lib);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(root);
}

#[test]
fn function_imports_use_exact_lookup_without_global_suffix_fallback() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "function-import-exact"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create exact function import fixture directory");
    let main = root.join("index.php");
    let lib = root.join("global.php");

    fs::write(
        &lib,
        r#"<?php
function fallback_only() {
    return "global";
}
"#,
    )
    .expect("write global fallback fixture");

    let source = r#"<?php
namespace App\Demo;

use function Vendor\Missing\fallback_only;
require 'global.php';

echo fallback_only();
"#;
    fs::write(&main, source).expect("write exact function import main fixture");

    let error = run_source_with_source_file(source, main.display().to_string()).unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "undefined function Vendor\\Missing\\fallback_only()"
    );

    let _ = fs::remove_file(lib);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(root);
}

#[test]
fn missing_imported_function_and_non_imported_namespaced_calls_report_distinct_runtime_names() {
    let imported_error = run_source(
        r#"<?php
namespace App\Demo;
use function Vendor\Missing\shared;
shared();
"#,
    )
    .unwrap_err();
    assert_eq!(imported_error.phase, Phase::Runtime);
    assert_eq!(imported_error.line, 4);
    assert_eq!(
        imported_error.message,
        "undefined function Vendor\\Missing\\shared()"
    );

    let fallback_error = run_source(
        r#"<?php
namespace App\Demo;
shared();
"#,
    )
    .unwrap_err();
    assert_eq!(fallback_error.phase, Phase::Runtime);
    assert_eq!(fallback_error.line, 3);
    assert_eq!(
        fallback_error.message,
        "undefined function App\\Demo\\shared()"
    );
}

#[test]
fn function_imports_reject_same_namespace_alias_conflicts() {
    let import_then_function = parse_error(
        r#"<?php
namespace App\Demo;
use function Vendor\Tools\label;
function label() {}
"#,
    );
    assert_eq!(import_then_function.phase, Phase::Parse);
    assert_eq!(import_then_function.line, 4);
    assert_eq!(
        import_then_function.message,
        "unsupported function declaration: function name conflicts with an imported function alias in the same namespace"
    );

    let function_then_import = parse_error(
        r#"<?php
namespace App\Demo;
function label() {}
use function Vendor\Tools\label;
"#,
    );
    assert_eq!(function_then_import.phase, Phase::Parse);
    assert_eq!(function_then_import.line, 4);
    assert_eq!(
        function_then_import.message,
        "unsupported function use declaration: imported function alias conflicts with an existing function declaration or import in the same namespace"
    );

    let duplicate_import = parse_error(
        r#"<?php
namespace App\Demo;
use function Vendor\Tools\label;
use function Other\Tools\label;
"#,
    );
    assert_eq!(duplicate_import.phase, Phase::Parse);
    assert_eq!(duplicate_import.line, 4);
    assert_eq!(
        duplicate_import.message,
        "unsupported function use declaration: imported function alias conflicts with an existing function declaration or import in the same namespace"
    );
}

#[test]
fn const_imports_resolve_aliases_and_keep_non_imported_fallback() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "const-imports"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create const import fixture directory");
    let main = root.join("index.php");
    let lib = root.join("constants.php");

    fs::write(
        &lib,
        r#"<?php
namespace Vendor\Values;

const PRIMARY = "vendor-primary";
const SECONDARY = "vendor-secondary";
"#,
    )
    .expect("write const import library fixture");

    let source = r#"<?php
namespace App\Demo;

use const Vendor\Values\PRIMARY as picked_value, Vendor\Values\SECONDARY;
require 'constants.php';

define("GLOBAL_ONLY", "global");
define("LOCAL_ONLY", "global-local");
const LOCAL_ONLY = "local";

echo picked_value, "\n";
echo SECONDARY, "\n";
echo LOCAL_ONLY, "\n";
echo GLOBAL_ONLY;
"#;
    fs::write(&main, source).expect("write const import main fixture");

    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        "vendor-primary\nvendor-secondary\nlocal\nglobal"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(lib);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(root);
}

#[test]
fn const_imports_use_exact_lookup_without_namespace_or_global_fallback() {
    let root = std::env::temp_dir().join(format!(
        "phpc-namespace-resolution-{}-{}",
        std::process::id(),
        "const-import-exact"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create exact const import fixture directory");
    let main = root.join("index.php");

    let source = r#"<?php
namespace App\Demo;

define("GLOBAL_ONLY", "global");
const GLOBAL_ONLY = "local";
use const Vendor\Missing\GLOBAL_ONLY as MISSING_ALIAS;

echo MISSING_ALIAS;
"#;
    fs::write(&main, source).expect("write exact const import main fixture");

    let error = run_source_with_source_file(source, main.display().to_string()).unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 8);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "undefined constant Vendor\\Missing\\GLOBAL_ONLY"
    );

    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(root);
}

#[test]
fn const_imports_reject_alias_conflicts_and_const_declarations() {
    let duplicate_import = parse_error(
        r#"<?php
namespace App\Demo;
use const Vendor\Tools\VALUE;
use const Other\Tools\VALUE;
"#,
    );
    assert_eq!(duplicate_import.phase, Phase::Parse);
    assert_eq!(duplicate_import.line, 4);
    assert_eq!(
        duplicate_import.message,
        "unsupported const use declaration: imported constant alias conflicts with an existing constant declaration or import in the same namespace"
    );

    let declaration_after_import = parse_error(
        r#"<?php
namespace App\Demo;
use const Vendor\Tools\VALUE;
const VALUE = 1;
"#,
    );
    assert_eq!(declaration_after_import.phase, Phase::Parse);
    assert_eq!(declaration_after_import.line, 4);
    assert_eq!(
        declaration_after_import.message,
        "unsupported const declaration: constant name conflicts with an imported constant alias in the same namespace"
    );

    let import_after_declaration = parse_error(
        r#"<?php
namespace App\Demo;
const VALUE = 1;
use const Vendor\Tools\VALUE;
"#,
    );
    assert_eq!(import_after_declaration.phase, Phase::Parse);
    assert_eq!(import_after_declaration.line, 4);
    assert_eq!(
        import_after_declaration.message,
        "unsupported const use declaration: imported constant alias conflicts with an existing constant declaration or import in the same namespace"
    );
}

#[test]
fn unbracketed_namespace_resolves_const_declarations_to_qualified_names() {
    let execution = run_source(
        r#"<?php
namespace Sodium;

class ParagonIE_Sodium_Compat {
    public const CRYPTO_AUTH_BYTES = 32;
}

const CRYPTO_AUTH_BYTES = ParagonIE_Sodium_Compat::CRYPTO_AUTH_BYTES;
const CRYPTO_SECRETBOX_KEYBYTES = 32;

echo defined("\\Sodium\\CRYPTO_AUTH_BYTES") ? "yes" : "no", "\n";
echo constant("\\Sodium\\CRYPTO_AUTH_BYTES"), "\n";
echo defined("Sodium\\CRYPTO_SECRETBOX_KEYBYTES") ? "yes" : "no", "\n";
echo defined("CRYPTO_AUTH_BYTES") ? "yes" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes\n32\nyes\nno");
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
use App\A, App\B;
"#,
            2,
            10,
            "unsupported multiple class use declaration: multiple simple class imports in one use declaration require import-list metadata, alias handling, namespace resolution, and native lowering",
        ),
        (
            r#"<?php
use App\Demo\{Service, Repository};
"#,
            2,
            14,
            "unsupported grouped use declaration: grouped class, function, and const imports are not implemented",
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
        "<?php\nnamespace App;\necho defined(\"\\\\PHP_VERSION_ID\");\n",
        "<?php\nnamespace App;\nuse Vendor\\Lib\\Tool;\necho Tool::class;\n",
        "<?php\nuse function strlen as len;\necho len(\"abc\");\n",
        "<?php\nuse const Vendor\\Values\\ANSWER;\necho ANSWER;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, LLVM_NAMESPACE_REJECTION);
    }
}

#[test]
fn generated_c_lowers_exact_imported_const_boundary() {
    let program = parse(
        r#"<?php
namespace App\Values;
const ANSWER = 42;
const LABEL = "answer";
use const App\Values\ANSWER as picked_number, App\Values\LABEL as picked_label;
use const PHP_VERSION_ID as runtime_version;
echo picked_label, "=", picked_number, "|", runtime_version;
"#,
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("42")
            && source.contains("80300")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "generated C should lower exact imported user and builtin constants:\n{source}"
    );
    assert!(
        !source.contains("global-constant lowering rejects")
            && !source.contains("namespace lowering rejects"),
        "{source}"
    );
}

#[test]
fn generated_c_rejects_missing_const_import_without_fallback() {
    let program = parse(
        "<?php\nnamespace App\\Values;\nconst ANSWER = 42;\nuse const Vendor\\Values\\ANSWER as missing_answer;\necho missing_answer;\n",
    )
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, ASSEMBLY_GLOBAL_CONSTANT_REJECTION);
}

#[test]
fn generated_c_rejects_imported_const_declaration_boundary_shapes() {
    let cases = [
        (
            "array const value",
            r#"<?php
namespace App\Values;
const ITEMS = [1];
use const App\Values\ITEMS as items;
echo items;
"#,
        ),
        (
            "dynamic const value",
            r#"<?php
namespace App\Values;
const COPIED = PHP_VERSION_ID;
use const App\Values\COPIED as copied;
echo copied;
"#,
        ),
        (
            "builtin const collision",
            r#"<?php
const PHP_VERSION_ID = 1;
use const PHP_VERSION_ID as version_id;
echo version_id;
"#,
        ),
    ];

    for (label, source) in cases {
        let program = parse(source).unwrap();
        let error = match emit_native_executable_c_source(&program) {
            Ok(source) => panic!("{label} should reject:\n{source}"),
            Err(error) => error,
        };

        assert_eq!(error.phase, Phase::Codegen, "{label}");
        assert_eq!(error.message, ASSEMBLY_GLOBAL_CONSTANT_REJECTION, "{label}");
    }
}

#[test]
fn generated_c_lowers_imported_runtime_builtin_function_boundary() {
    let program = parse("<?php\nuse function strlen as len;\necho len(\"abc\");\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "imported runtime builtin aliases should lower through runtime callable source calls:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_FUNCTION_CALL_REJECTION),
        "supported imported runtime builtin aliases should not hit the old metadata boundary:\n{source}"
    );
}

#[test]
fn generated_c_rejects_qualified_imported_type_builtin_without_exact_user_function() {
    let program =
        parse("<?php\nuse function Vendor\\Missing\\is_int as imported_is_int;\necho imported_is_int(1);\n")
            .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, ASSEMBLY_FUNCTION_CALL_REJECTION);
}

#[test]
fn assembly_lowering_rejects_namespace_context_before_backend_execution() {
    let error = emit_asm_source("<?php\nnamespace App;\necho \"ok\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_NAMESPACE_REJECTION);
}
