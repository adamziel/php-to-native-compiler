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
        "unsupported call spl_autoload_register(): callback argument must be closure, string, array callable, or invokable object in the current subset, got int"
    );

    let non_bool_throw = run_source("<?php\nspl_autoload_register('Loader', 1);\n").unwrap_err();
    assert_eq!(non_bool_throw.phase, Phase::Runtime);
    assert_eq!(non_bool_throw.line, 2);
    assert_eq!(non_bool_throw.column, 1);
    assert_eq!(
        non_bool_throw.message,
        "unsupported call spl_autoload_register(): argument #2 must be bool in the current subset, got int"
    );

    let non_invokable_object = run_source(
        r#"<?php
class LoaderWithoutInvoke {}
$loader = new LoaderWithoutInvoke();
spl_autoload_register($loader);
"#,
    )
    .unwrap_err();
    assert_eq!(non_invokable_object.phase, Phase::Runtime);
    assert_eq!(non_invokable_object.line, 4);
    assert_eq!(non_invokable_object.column, 1);
    assert_eq!(
        non_invokable_object.message,
        "unsupported call spl_autoload_register(): object callback LoaderWithoutInvoke must define public non-static __invoke($name) in the current subset"
    );

    let functions_arity = run_source("<?php\nspl_autoload_functions(1);\n").unwrap_err();
    assert_eq!(functions_arity.phase, Phase::Runtime);
    assert_eq!(functions_arity.line, 2);
    assert_eq!(functions_arity.column, 1);
    assert_eq!(
        functions_arity.message,
        "arity mismatch for spl_autoload_functions(): expected 0 argument(s), got 1"
    );

    let unregister_non_callable = run_source("<?php\nspl_autoload_unregister(42);\n").unwrap_err();
    assert_eq!(unregister_non_callable.phase, Phase::Runtime);
    assert_eq!(unregister_non_callable.line, 2);
    assert_eq!(unregister_non_callable.column, 1);
    assert_eq!(
        unregister_non_callable.message,
        "unsupported call spl_autoload_unregister(): callback argument must be closure, string, array callable, or invokable object in the current subset, got int"
    );

    let call_non_string = run_source("<?php\nspl_autoload_call(42);\n").unwrap_err();
    assert_eq!(call_non_string.phase, Phase::Runtime);
    assert_eq!(call_non_string.line, 2);
    assert_eq!(call_non_string.column, 1);
    assert_eq!(
        call_non_string.message,
        "unsupported call spl_autoload_call(): class name argument must be string in the current subset, got int"
    );

    let extensions_arity =
        run_source("<?php\nspl_autoload_extensions('.php', '.inc');\n").unwrap_err();
    assert_eq!(extensions_arity.phase, Phase::Runtime);
    assert_eq!(extensions_arity.line, 2);
    assert_eq!(extensions_arity.column, 1);
    assert_eq!(
        extensions_arity.message,
        "arity mismatch for spl_autoload_extensions(): expected 0 to 1 argument(s), got 2"
    );

    let extensions_non_string = run_source("<?php\nspl_autoload_extensions([]);\n").unwrap_err();
    assert_eq!(extensions_non_string.phase, Phase::Runtime);
    assert_eq!(extensions_non_string.line, 2);
    assert_eq!(extensions_non_string.column, 1);
    assert_eq!(
        extensions_non_string.message,
        "unsupported call spl_autoload_extensions(): file_extensions argument must be string or null in the current subset, got array"
    );

    let default_autoload_non_string = run_source("<?php\nspl_autoload(42);\n").unwrap_err();
    assert_eq!(default_autoload_non_string.phase, Phase::Runtime);
    assert_eq!(default_autoload_non_string.line, 2);
    assert_eq!(default_autoload_non_string.column, 1);
    assert_eq!(
        default_autoload_non_string.message,
        "unsupported call spl_autoload(): class name argument must be string in the current subset, got int"
    );

    let default_autoload_extension_non_string =
        run_source("<?php\nspl_autoload('Box', []);\n").unwrap_err();
    assert_eq!(default_autoload_extension_non_string.phase, Phase::Runtime);
    assert_eq!(default_autoload_extension_non_string.line, 2);
    assert_eq!(default_autoload_extension_non_string.column, 1);
    assert_eq!(
        default_autoload_extension_non_string.message,
        "unsupported call spl_autoload(): file_extensions argument must be string or null in the current subset, got array"
    );
}

#[test]
fn spl_autoload_extensions_tracks_request_local_extension_string() {
    let execution = run_source(
        r#"<?php
echo spl_autoload_extensions(), "\n";
echo spl_autoload_extensions(".php,.inc"), "\n";
echo spl_autoload_extensions(), "\n";
echo spl_autoload_extensions(null), "\n";
$call = "spl_autoload_extensions";
echo $call(".class.php"), "\n";
echo spl_autoload_extensions();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        ".inc,.php\n.php,.inc\n.php,.inc\n.php,.inc\n.class.php\n.class.php"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_autoload_probes_local_files_through_current_extension_registry() {
    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-default-spl-autoload-{}", std::process::id()));
    fs::create_dir_all(fixture_dir.join("acme")).unwrap();
    let class_path = fixture_dir.join("wp_loader.class.inc");
    fs::write(
        &class_path,
        r#"<?php
class Wp_Loader {
    public $name = "direct";
}
"#,
    )
    .unwrap();
    let namespace_path = fixture_dir.join("acme").join("plugin.inc");
    fs::write(
        &namespace_path,
        r#"<?php
namespace Acme;
class Plugin {
    public $name = "namespaced";
}
"#,
    )
    .unwrap();
    let registered_path = fixture_dir.join("registeredbox.autoload.inc");
    fs::write(
        &registered_path,
        r#"<?php
class RegisteredBox {
    public $name = "registered";
}
"#,
    )
    .unwrap();

    let source = r#"<?php
spl_autoload_extensions(".class.inc,.inc");
spl_autoload("Wp_Loader");
$box = new Wp_Loader();
echo $box->name, "\n";

$call = "spl_autoload";
$call("Acme\\Plugin", null);
echo class_exists("Acme\\Plugin", false) ? "namespace\n" : "missing-namespace\n";

spl_autoload_extensions(".autoload.inc");
spl_autoload_register("spl_autoload");
$registered = new RegisteredBox();
echo $registered->name;
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(execution.stdout, "direct\nnamespace\nregistered");
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(class_path);
    let _ = fs::remove_file(namespace_path);
    let _ = fs::remove_file(registered_path);
    let _ = fs::remove_dir(fixture_dir.join("acme"));
    let _ = fs::remove_dir(fixture_dir);
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
fn array_callable_autoload_callbacks_load_class_like_dependencies() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-array-callables-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let class_include_path = fixture_dir.join("StaticLoadedBox.inc");
    fs::write(
        &class_include_path,
        r#"<?php
class StaticLoadedBox {
    public $name = "static";
}
"#,
    )
    .unwrap();
    let interface_include_path = fixture_dir.join("ObjectLoadedContract.inc");
    fs::write(
        &interface_include_path,
        r#"<?php
interface ObjectLoadedContract {
    public function boot();
}
"#,
    )
    .unwrap();
    let plugin_include_path = fixture_dir.join("ObjectPlugin.inc");
    fs::write(
        &plugin_include_path,
        r#"<?php
class ObjectPlugin implements ObjectLoadedContract {
    use ObjectLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
"#,
    )
    .unwrap();
    let trait_include_path = fixture_dir.join("ObjectLoadedTrait.inc");
    fs::write(
        &trait_include_path,
        r#"<?php
trait ObjectLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
class StaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

class ObjectLoader {
    public function load($name) {
        if ($name === "StaticLoadedBox") {
            return false;
        }
        echo "object:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

$loader = new ObjectLoader();
spl_autoload_register(array("StaticLoader", "load"));
spl_autoload_register(array($loader, "load"), true, true);

echo class_exists("StaticLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("ObjectLoadedContract") ? "interface\n" : "missing-interface\n";
require_once __DIR__ . "/ObjectPlugin.inc";
$plugin = new ObjectPlugin();
echo $plugin->boot(), "\n";
echo trait_exists("ObjectLoadedTrait", false) ? "trait" : "missing-trait";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "static:StaticLoadedBox\nclass\nobject:ObjectLoadedContract\ninterface\nobject:ObjectLoadedTrait\nhook:ObjectPlugin\ntrait"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(class_include_path);
    let _ = fs::remove_file(interface_include_path);
    let _ = fs::remove_file(plugin_include_path);
    let _ = fs::remove_file(trait_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn static_method_string_autoload_callbacks_load_class_like_dependencies() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-static-method-string-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let class_include_path = fixture_dir.join("StringLoadedBox.inc");
    fs::write(
        &class_include_path,
        r#"<?php
class StringLoadedBox {
    public $name = "string";
}
"#,
    )
    .unwrap();
    let interface_include_path = fixture_dir.join("StringLoadedContract.inc");
    fs::write(
        &interface_include_path,
        r#"<?php
interface StringLoadedContract {
    public function boot();
}
"#,
    )
    .unwrap();
    let plugin_include_path = fixture_dir.join("StringPlugin.inc");
    fs::write(
        &plugin_include_path,
        r#"<?php
class StringPlugin implements StringLoadedContract {
    use StringLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
"#,
    )
    .unwrap();
    let trait_include_path = fixture_dir.join("StringLoadedTrait.inc");
    fs::write(
        &trait_include_path,
        r#"<?php
trait StringLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
class StaticStringLoader {
    public static function load($name) {
        echo "string-static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

spl_autoload_register("StaticStringLoader::load");

echo class_exists("StringLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("StringLoadedContract") ? "interface\n" : "missing-interface\n";
require_once __DIR__ . "/StringPlugin.inc";
$plugin = new StringPlugin();
echo $plugin->boot(), "\n";
echo trait_exists("StringLoadedTrait", false) ? "trait" : "missing-trait";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "string-static:StringLoadedBox\nclass\nstring-static:StringLoadedContract\ninterface\nstring-static:StringLoadedTrait\nhook:StringPlugin\ntrait"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(class_include_path);
    let _ = fs::remove_file(interface_include_path);
    let _ = fs::remove_file(plugin_include_path);
    let _ = fs::remove_file(trait_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn invokable_object_autoload_callbacks_load_class_like_dependencies() {
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-autoload-invokable-object-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).unwrap();
    let class_include_path = fixture_dir.join("InvokeLoadedBox.inc");
    fs::write(
        &class_include_path,
        r#"<?php
class InvokeLoadedBox {
    public $name = "invoke";
}
"#,
    )
    .unwrap();
    let interface_include_path = fixture_dir.join("InvokeLoadedContract.inc");
    fs::write(
        &interface_include_path,
        r#"<?php
interface InvokeLoadedContract {
    public function boot();
}
"#,
    )
    .unwrap();
    let plugin_include_path = fixture_dir.join("InvokePlugin.inc");
    fs::write(
        &plugin_include_path,
        r#"<?php
class InvokePlugin implements InvokeLoadedContract {
    use InvokeLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
"#,
    )
    .unwrap();
    let trait_include_path = fixture_dir.join("InvokeLoadedTrait.inc");
    fs::write(
        &trait_include_path,
        r#"<?php
trait InvokeLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
class InvokeLoader {
    public function __invoke($name) {
        echo "invoke:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

$loader = new InvokeLoader();
spl_autoload_register($loader);

echo class_exists("InvokeLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("InvokeLoadedContract") ? "interface\n" : "missing-interface\n";
require_once __DIR__ . "/InvokePlugin.inc";
$plugin = new InvokePlugin();
echo $plugin->boot(), "\n";
echo trait_exists("InvokeLoadedTrait", false) ? "trait" : "missing-trait";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "invoke:InvokeLoadedBox\nclass\ninvoke:InvokeLoadedContract\ninterface\ninvoke:InvokeLoadedTrait\nhook:InvokePlugin\ntrait"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(class_include_path);
    let _ = fs::remove_file(interface_include_path);
    let _ = fs::remove_file(plugin_include_path);
    let _ = fs::remove_file(trait_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn spl_autoload_call_invokes_registered_callbacks_for_manual_class_like_loads() {
    let fixture_dir =
        std::env::temp_dir().join(format!("phpc-autoload-manual-call-{}", std::process::id()));
    fs::create_dir_all(&fixture_dir).unwrap();
    let class_include_path = fixture_dir.join("ManualLoadedBox.inc");
    fs::write(
        &class_include_path,
        r#"<?php
class ManualLoadedBox {
    public $name = "manual";
}
"#,
    )
    .unwrap();
    let interface_include_path = fixture_dir.join("ManualLoadedContract.inc");
    fs::write(
        &interface_include_path,
        r#"<?php
interface ManualLoadedContract {
    public function boot();
}
"#,
    )
    .unwrap();
    let trait_include_path = fixture_dir.join("ManualLoadedTrait.inc");
    fs::write(
        &trait_include_path,
        r#"<?php
trait ManualLoadedTrait {
    public function hook() {
        return "hook";
    }
}
"#,
    )
    .unwrap();

    let source = r#"<?php
function ManualLoader($name) {
    echo "manual:", $name, "\n";
    require_once __DIR__ . "/" . $name . ".inc";
}

class ManualStaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

spl_autoload_register("ManualLoader");
spl_autoload_register(array("ManualStaticLoader", "load"));

$result = spl_autoload_call("ManualLoadedBox");
echo is_null($result) ? "null\n" : "not-null\n";
echo class_exists("ManualLoadedBox", false) ? "class\n" : "missing-class\n";

spl_autoload_unregister("ManualLoader");
spl_autoload_call("ManualLoadedContract");
echo interface_exists("ManualLoadedContract", false) ? "interface\n" : "missing-interface\n";

spl_autoload_call("ManualLoadedTrait");
echo trait_exists("ManualLoadedTrait", false) ? "trait" : "missing-trait";
"#;

    let execution =
        run_source_with_source_file(source, fixture_dir.join("main.php").display().to_string())
            .unwrap();
    assert_eq!(
        execution.stdout,
        "manual:ManualLoadedBox\nnull\nclass\nstatic:ManualLoadedContract\ninterface\nstatic:ManualLoadedTrait\ntrait"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(class_include_path);
    let _ = fs::remove_file(interface_include_path);
    let _ = fs::remove_file(trait_include_path);
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn spl_autoload_functions_and_unregister_manage_bounded_callback_lifecycle() {
    let execution = run_source(
        r#"<?php
function FirstLoader($name) {
    echo "first:", $name, "\n";
}

function OtherLoader($name) {
    echo "other:", $name, "\n";
}

class StaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
    }
}

class ObjectLoader {
    public function load($name) {
        echo "object:", $name, "\n";
    }

    public function __invoke($name) {
        echo "invoke:", $name, "\n";
    }
}

$loader = new ObjectLoader();
echo count(spl_autoload_functions()), "\n";
spl_autoload_register("FirstLoader");
spl_autoload_register(array("StaticLoader", "load"));
spl_autoload_register(array($loader, "load"));
spl_autoload_register($loader, true, true);

$callbacks = spl_autoload_functions();
echo count($callbacks), "\n";
echo is_object($callbacks[0]) ? get_class($callbacks[0]) : "not-object", "\n";
echo $callbacks[1], "\n";
echo $callbacks[2][0], "::", $callbacks[2][1], "\n";
echo get_class($callbacks[3][0]), "::", $callbacks[3][1], "\n";

echo class_exists("MissingOne") ? "loaded\n" : "missing\n";
echo spl_autoload_unregister($loader) ? "removed-invoke\n" : "missing-invoke\n";
echo spl_autoload_unregister(array("StaticLoader", "load")) ? "removed-static\n" : "missing-static\n";
echo spl_autoload_unregister("OtherLoader") ? "removed-missing\n" : "missing-callback\n";

$callbacks = spl_autoload_functions();
echo count($callbacks), "\n";
echo class_exists("MissingTwo") ? "loaded" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0\n4\nObjectLoader\nFirstLoader\nStaticLoader::load\nObjectLoader::load\ninvoke:MissingOne\nfirst:MissingOne\nstatic:MissingOne\nobject:MissingOne\nmissing\nremoved-invoke\nremoved-static\nmissing-callback\n2\nfirst:MissingTwo\nobject:MissingTwo\nmissing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_spl_autoload_register_until_native_autoloading_exists() {
    let autoload_error = emit_ir_source("<?php\nspl_autoload('MissingBox');\n").unwrap_err();
    assert_eq!(autoload_error.phase, Phase::Codegen);
    assert_eq!(autoload_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nspl_autoload_register('MissingAutoloader');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let functions_error = emit_ir_source("<?php\nspl_autoload_functions();\n").unwrap_err();
    assert_eq!(functions_error.phase, Phase::Codegen);
    assert_eq!(functions_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let extensions_error = emit_ir_source("<?php\nspl_autoload_extensions();\n").unwrap_err();
    assert_eq!(extensions_error.phase, Phase::Codegen);
    assert_eq!(extensions_error.message, LLVM_FUNCTION_CALL_REJECTION);

    let unregister_error =
        emit_ir_source("<?php\nspl_autoload_unregister('MissingAutoloader');\n").unwrap_err();
    assert_eq!(unregister_error.phase, Phase::Codegen);
    assert_eq!(unregister_error.message, LLVM_FUNCTION_CALL_REJECTION);
}
