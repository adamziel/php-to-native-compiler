use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::{run_source, run_source_with_source_file};

fn php_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("phpc-opcache-{name}-{nanos}.php"))
}

#[test]
fn opcache_configuration_status_and_ini_registry_are_bounded() {
    let execution = run_source(
        r#"<?php
ini_set("opcache.enable_cli", "1");
ini_set("opcache.interned_strings_buffer", "16");
ini_set("opcache.jit_prof_threshold", 1 / 128);
$config = opcache_get_configuration();
$status = opcache_get_status();
var_dump($config["directives"]["opcache.enable"]);
var_dump($config["directives"]["opcache.enable_cli"]);
var_dump($status["interned_strings_usage"]["used_memory"] + $status["interned_strings_usage"]["free_memory"]);
var_dump($status["interned_strings_usage"]["buffer_size"]);
var_dump($config["directives"]["opcache.jit_prof_threshold"]);
var_dump(array_diff_key(ini_get_all("zend opcache"), $config["directives"]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nint(16777216)\nint(16777216)\nfloat(0.0078125)\narray(0) {\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn opcache_script_cache_queries_are_bounded_noops() {
    let main = unique_temp_path("main");
    let cached = unique_temp_path("cached");
    fs::write(&main, "<?php\n").unwrap();
    fs::write(&cached, "<?php return 1;\n").unwrap();

    let source = format!(
        r#"<?php
ini_set("opcache.enable_cli", "1");
$cached = '{}';
var_dump(opcache_is_script_cached(__FILE__));
var_dump(opcache_is_script_cached(__DIR__ . "/missing.php"));
var_dump(opcache_is_script_cached_in_file_cache($cached));
var_dump(opcache_compile_file($cached));
var_dump(opcache_is_script_cached_in_file_cache($cached));
var_dump(opcache_invalidate($cached, true));
var_dump(opcache_is_script_cached_in_file_cache($cached));
"#,
        php_string(cached.to_str().unwrap())
    );

    let execution = run_source_with_source_file(&source, main.to_str().unwrap()).unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(main);
    let _ = fs::remove_file(cached);
}

#[test]
fn opcache_enable_cannot_be_reenabled_after_runtime_disable() {
    let execution = run_source(
        r#"<?php
ini_set("opcache.enable_cli", "1");
ini_set("opcache.enable", "0");
ini_set("opcache.enable", "1");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: Zend OPcache can't be temporarily enabled (it may be only disabled until the end of request) in Command line code on line 4\n"
    );
    assert_eq!(execution.exit_code, 0);
}
