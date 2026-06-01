use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

use php_compiler::run_source;

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("environment lock is not poisoned")
}

fn set_env_var(name: &str, value: &str) {
    env::set_var(name, value);
}

fn restore_env_var(name: &str, previous: Option<std::ffi::OsString>) {
    if let Some(value) = previous {
        env::set_var(name, value);
    } else {
        env::remove_var(name);
    }
}

#[test]
fn reflection_function_variadic_constructor_and_has_method_rows() {
    let execution = run_source(
        r#"<?php
function test1($args) {}
function test2(...$args) {}
function test3($arg, ...$args) {}

var_dump((new ReflectionFunction('test1'))->isVariadic());
var_dump((new ReflectionFunction('test2'))->isVariadic());
var_dump((new ReflectionFunction('test3'))->isVariadic());

class NewCtor {
    function __construct() {}
}

class ExtendsNewCtor extends NewCtor {
}

$classes = array('NewCtor', 'ExtendsNewCtor');
foreach ($classes as $class) {
    $rc = new ReflectionClass($class);
    $rm = $rc->getConstructor();
    if ($rm != null) {
        echo "Constructor of $class: " . $rm->getName() . "\n";
    } else {
        echo "No constructor for $class\n";
    }
}

class C {
    function f() {}
}

$rc = new ReflectionClass("C");
echo "Check invalid params:\n";
var_dump($rc->hasMethod(1));
var_dump($rc->hasMethod(1.5));
var_dump($rc->hasMethod(true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "Constructor of NewCtor: __construct\n",
            "Constructor of ExtendsNewCtor: __construct\n",
            "Check invalid params:\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reflection_function_is_disabled_matches_disabled_function_phpt_row() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");
    set_env_var("PHPC_PHPT_INI_FLAGS", "-d disable_functions=is_file");

    let execution = run_source(
        r#"<?php
try {
    $rf = new ReflectionFunction('is_file');
    var_dump($rf->isDisabled());
} catch (ReflectionException $e) {
    echo $e->getMessage(), "\n";
}

$rf = new ReflectionFunction('is_string');
var_dump($rf->isDisabled());
"#,
    )
    .unwrap();
    let stdout = execution.stdout.clone();
    let stderr = execution.stderr.clone();
    let exit_code = execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert!(stdout.starts_with("Function is_file() does not exist\n"));
    assert!(stdout.contains(
        "Deprecated: Method ReflectionFunction::isDisabled() is deprecated since 8.0, as ReflectionFunction can no longer be constructed for disabled functions"
    ));
    assert!(stdout.ends_with("bool(false)\n"));
    assert_eq!(stderr, "");
    assert_eq!(exit_code, 0);
}
