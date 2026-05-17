use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source, run_source_with_source_file};
use std::fs;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn spl_autoload_register_accepts_current_callback_shapes_without_invoking_them() {
    let execution = run_source(
        r#"<?php
$called = "no";
echo spl_autoload_register(function ($class) use ($called) {
    echo "called";
    return false;
}) ? "1" : "0";
echo "|", $called, "\n";

$arrow_called = "no";
echo spl_autoload_register(fn ($class) => false) ? "1" : "0";
echo "|", $arrow_called, "\n";

$call = "spl_autoload_register";
function MissingAutoloader($class) {
    return false;
}
echo $call("MissingAutoloader", true, false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|no\n1|no\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_autoload_register_reports_current_argument_boundaries() {
    let non_callable = run_source("<?php\nspl_autoload_register(42);\n").unwrap_err();
    assert_eq!(non_callable.phase, Phase::Runtime);
    assert_eq!(non_callable.line, 2);
    assert_eq!(non_callable.column, 1);
    assert_eq!(
        non_callable.message,
        "unsupported call spl_autoload_register(): callback argument must be closure or string in the current subset, got int"
    );

    let non_bool_throw = run_source("<?php\nspl_autoload_register('Loader', 1);\n").unwrap_err();
    assert_eq!(non_bool_throw.phase, Phase::Runtime);
    assert_eq!(non_bool_throw.line, 2);
    assert_eq!(non_bool_throw.column, 1);
    assert_eq!(
        non_bool_throw.message,
        "unsupported call spl_autoload_register(): argument #2 must be bool in the current subset, got int"
    );
}

#[test]
fn class_and_interface_exists_invoke_string_autoload_callbacks() {
    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-autoload-metadata-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let include_path = fixture_dir.join("autoloaded_metadata.inc");
    fs::write(
        &include_path,
        "<?php\nclass LoadedBox {}\ninterface LoadedContract {}\n",
    )
    .unwrap();

    let source = r#"<?php
function Loader($class) {
    echo "load:", $class, "\n";
    require_once __DIR__ . "/autoloaded_metadata.inc";
}

spl_autoload_register("Loader");

echo class_exists("MissingBox", false) ? "false-loaded\n" : "false-skip\n";
echo class_exists("LoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("LoadedContract") ? "interface" : "missing-interface";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "false-skip\nload:LoadedBox\nclass\ninterface"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn new_expressions_invoke_string_autoload_callbacks() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-new-expression-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let named_include_path = fixture_dir.join("LoadedBox.inc");
    fs::write(
        &named_include_path,
        r#"<?php
class LoadedBox {
    public $name;

    public function __construct($name = "named") {
        $this->name = $name;
    }
}
"#,
    )
    .unwrap();
    let dynamic_include_path = fixture_dir.join("DynamicBox.inc");
    fs::write(
        &dynamic_include_path,
        r#"<?php
class DynamicBox {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
function LoadClass($class) {
    echo "load:", $class, "\n";
    require_once __DIR__ . "/" . $class . ".inc";
}

spl_autoload_register("LoadClass");

$box = new LoadedBox();
echo $box->name, "\n";

$class = "DynamicBox";
$dynamic = new $class("dynamic");
echo $dynamic->name;
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "load:LoadedBox\nnamed\nload:DynamicBox\ndynamic"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(named_include_path);
    let _ = fs::remove_file(dynamic_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn included_class_declarations_autoload_missing_extends_and_implements_dependencies() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-class-dependencies-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let child_include_path = fixture_dir.join("ChildPlugin.inc");
    fs::write(
        &child_include_path,
        r#"<?php
class ChildPlugin extends LoadedBase implements LoadedContract {
    public function label() {
        return $this->name;
    }

    public function boot() {
        return "boot";
    }
}
"#,
    )
    .unwrap();
    let parent_include_path = fixture_dir.join("LoadedBase.inc");
    fs::write(
        &parent_include_path,
        r#"<?php
class LoadedBase {
    public $name;

    public function __construct($name = "base") {
        $this->name = $name;
    }
}
"#,
    )
    .unwrap();
    let interface_include_path = fixture_dir.join("LoadedContract.inc");
    fs::write(
        &interface_include_path,
        r#"<?php
interface LoadedContract extends BaseContract {
    public function boot();
}
"#,
    )
    .unwrap();
    let parent_interface_include_path = fixture_dir.join("BaseContract.inc");
    fs::write(
        &parent_interface_include_path,
        r#"<?php
interface BaseContract {
    public function label();
}
"#,
    )
    .unwrap();

    let source = r#"<?php
function LoadDependency($name) {
    require_once __DIR__ . "/" . $name . ".inc";
}

spl_autoload_register("LoadDependency");

require_once __DIR__ . "/ChildPlugin.inc";

$plugin = new ChildPlugin("wp");
echo get_parent_class($plugin), "\n";
echo $plugin->name, ":", $plugin->label(), ":", $plugin->boot(), "\n";
echo is_a($plugin, "BaseContract") ? "base-contract\n" : "missing-base\n";
echo is_a($plugin, "LoadedContract") ? "loaded-contract" : "missing-loaded";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "LoadedBase\nwp:wp:boot\nbase-contract\nloaded-contract"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(child_include_path);
    let _ = fs::remove_file(parent_include_path);
    let _ = fs::remove_file(interface_include_path);
    let _ = fs::remove_file(parent_interface_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn trait_exists_and_included_class_declarations_autoload_missing_trait_dependencies() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-trait-dependencies-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let probe_include_path = fixture_dir.join("AutoloadedProbe.inc");
    fs::write(
        &probe_include_path,
        r#"<?php
trait AutoloadedProbe {
    public function probe() {
        return "probe";
    }
}
"#,
    )
    .unwrap();
    let plugin_include_path = fixture_dir.join("Plugin.inc");
    fs::write(
        &plugin_include_path,
        r#"<?php
class Plugin {
    use LoadedHook;

    public function boot() {
        return $this->hook();
    }
}
"#,
    )
    .unwrap();
    let trait_include_path = fixture_dir.join("LoadedHook.inc");
    fs::write(
        &trait_include_path,
        r#"<?php
trait LoadedHook {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
function LoadObjectDependency($name) {
    echo "load:", $name, "\n";
    require_once __DIR__ . "/" . $name . ".inc";
}

spl_autoload_register("LoadObjectDependency");

echo trait_exists("AutoloadedProbe", false) ? "false-loaded\n" : "false-skip\n";
echo trait_exists("AutoloadedProbe") ? "probe-loaded\n" : "probe-missing\n";

require_once __DIR__ . "/Plugin.inc";

$plugin = new Plugin();
echo $plugin->boot(), "\n";
echo trait_exists("LoadedHook", false) ? "hook-loaded" : "hook-missing";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "false-skip\nload:AutoloadedProbe\nprobe-loaded\nload:LoadedHook\nhook:Plugin\nhook-loaded"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(probe_include_path);
    let _ = fs::remove_file(plugin_include_path);
    let _ = fs::remove_file(trait_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn emit_ir_rejects_direct_spl_autoload_register_until_native_autoloading_exists() {
    let error = emit_ir_source("<?php\nspl_autoload_register('MissingAutoloader');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
