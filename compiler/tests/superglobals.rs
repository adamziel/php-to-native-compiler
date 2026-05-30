use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::interpreter::{run_program_with_options, RunOptions};
use php_compiler::parse;
use php_compiler::run_source;
use std::path::Path;
use std::process::{Command, Output};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_REQUEST_SUPERGLOBAL_REJECTION: &str = "LLVM request-superglobal lowering rejects $_SERVER, $_COOKIE, $_GET, $_POST, $_REQUEST, $_FILES, and $_SESSION until native request-state storage, SAPI population, variables_order policy, upload metadata, session storage, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded request superglobal behavior";

fn llvm_request_superglobal_array_key_consumer_rejection(subject: &str) -> String {
    format!(
        "LLVM request-superglobal lowering rejects array-key request operand for {subject} because request-backed ordinary array keys need ordered key expression evaluation, PHP array-key coercion diagnostics, missing-array recovery values, write/unset/reference ordering, root symbol-table reconciliation, references/copy-on-write, and exact PHP array-key diagnostics; phpc run handles current bounded request superglobal behavior"
    )
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn server_superglobal_is_materialized_with_current_request_defaults() {
    let execution = run_source(
        r#"<?php
echo is_array($_SERVER) ? "array" : "missing";
echo "|";
echo $_SERVER["REQUEST_URI"];
echo "|";
echo $_SERVER["HTTP_HOST"];
echo "|";
echo $_SERVER["PHP_SELF"];
echo "|";
echo $_SERVER["SCRIPT_FILENAME"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|/|localhost|/index.php|/index.php");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_is_available_as_current_cli_runtime_constant() {
    let execution = run_source(
        r#"<?php
echo PHP_SAPI;
echo "|";
echo defined("PHP_SAPI") ? "defined" : "missing";
echo "|";
echo constant("PHP_SAPI");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cli|defined|cli");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_binary_is_available_as_current_cli_runtime_constant() {
    let execution = run_source(
        r#"<?php
echo defined("PHP_BINARY") ? "defined" : "missing";
echo "|";
echo PHP_BINARY !== "" ? "non-empty" : "empty";
echo "|";
echo constant("PHP_BINARY") === PHP_BINARY ? "same" : "different";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "defined|non-empty|same");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_name_returns_current_cli_runtime_sapi() {
    let execution = run_source(
        r#"<?php
echo php_sapi_name();
echo "|";
$call = "php_sapi_name";
echo function_exists($call) ? "exists" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cli|exists|callable|cli");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_name_rejects_arguments() {
    let error = runtime_error(
        r#"<?php
echo php_sapi_name("cgi-fcgi");
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for php_sapi_name(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_php_sapi_name_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("php_sapi_name") ? "1" : "0";
echo is_callable("php_sapi_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho php_sapi_name();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn server_superglobal_routes_function_scope_reads_and_writes_to_root() {
    let execution = run_source(
        r#"<?php
function fix_server_vars() {
    $_SERVER = array_merge(["SERVER_SOFTWARE" => "", "REQUEST_URI" => ""], $_SERVER);
    if (empty($_SERVER["REQUEST_URI"])) {
        $_SERVER["REQUEST_URI"] = "/";
    }
    $_SERVER["HTTP_HOST"] = "example.test";
}

fix_server_vars();
echo $_SERVER["SERVER_SOFTWARE"];
echo "|";
echo $_SERVER["REQUEST_URI"];
echo "|";
echo $_SERVER["HTTP_HOST"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "phpc|/|example.test");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn cookie_superglobal_is_materialized_as_empty_auto_global_array() {
    let execution = run_source(
        r#"<?php
echo is_array($_COOKIE) ? "array" : "missing";
echo "|";
echo isset($_COOKIE["wordpress_test_cookie"]) ? "cookie" : "empty";

function seed_cookie() {
    $_COOKIE["wordpress_test_cookie"] = "WP Cookie check";
}

seed_cookie();
echo "|";
echo $_COOKIE["wordpress_test_cookie"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|empty|WP Cookie check");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn cookie_superglobal_parses_explicit_cli_cookie_seed() {
    let program = parse(
        r#"<?php
echo $_SERVER["HTTP_COOKIE"];
echo "|";
echo $_COOKIE["wordpress_test_cookie"];
echo "|";
echo $_COOKIE["logged_in"];
echo "|";
echo $_COOKIE["settings"]["theme"];
echo "|";
echo $_COOKIE["dotted_name"];
echo "|";
echo isset($_REQUEST["wordpress_test_cookie"]) ? "request-cookie" : "request-empty";
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            cookie_header: Some(
                "wordpress_test_cookie=WP+Cookie+check; logged_in=user%7Ctoken; settings[theme]=classic; dotted.name=value"
                    .to_string(),
            ),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wordpress_test_cookie=WP+Cookie+check; logged_in=user%7Ctoken; settings[theme]=classic; dotted.name=value|WP Cookie check|user|token|classic|value|request-empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_superglobals_are_materialized_as_empty_auto_global_arrays() {
    let execution = run_source(
        r#"<?php
echo is_array($_GET) ? "get-array" : "get-missing";
echo "|";
echo is_array($_POST) ? "post-array" : "post-missing";
echo "|";
echo is_array($_REQUEST) ? "request-array" : "request-missing";
echo "|";
echo isset($_GET["preview"]) ? "preview" : "get-empty";
echo "|";
echo isset($_POST["action"]) ? "action" : "post-empty";
echo "|";
echo isset($_REQUEST["name"]) ? "name" : "request-empty";

function seed_request_bags() {
    $_GET["preview"] = "true";
    $_POST["action"] = "save";
    $_REQUEST["name"] = $_GET["preview"] . ":" . $_POST["action"];
}

seed_request_bags();
echo "|";
echo $_GET["preview"];
echo "|";
echo $_POST["action"];
echo "|";
echo $_REQUEST["name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "get-array|post-array|request-array|get-empty|post-empty|request-empty|true|save|true:save"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_superglobals_parse_flat_query_and_form_body_options() {
    let program = parse(
        r#"<?php
echo $_SERVER["REQUEST_METHOD"];
echo "|";
echo $_SERVER["QUERY_STRING"];
echo "|";
echo $_SERVER["CONTENT_TYPE"];
echo "|";
echo $_SERVER["CONTENT_LENGTH"];
echo "|";
echo $_GET["preview"];
echo "|";
echo $_GET["name"];
echo "|";
echo $_GET["encoded"];
echo "|";
echo $_POST["action"];
echo "|";
echo $_POST["space"];
echo "|";
echo $_REQUEST["preview"];
echo "|";
echo $_REQUEST["action"];
echo "|";
echo file_get_contents("php://input");
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            query_string: Some("preview=true&name=WordPress%20Core&encoded=a%2Bb".to_string()),
            request_body: Some("action=save&space=hello+world".to_string()),
            request_method: Some("POST".to_string()),
            content_type: Some("application/x-www-form-urlencoded; charset=UTF-8".to_string()),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "POST|preview=true&name=WordPress%20Core&encoded=a%2Bb|application/x-www-form-urlencoded; charset=UTF-8|29|true|WordPress Core|a+b|save|hello world|true|save|action=save&space=hello+world"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_superglobals_keep_body_out_of_post_without_form_content_type() {
    let program = parse(
        r#"<?php
echo file_get_contents("php://input");
echo "|";
echo isset($_POST["action"]) ? "post" : "no-post";
echo "|";
echo isset($_REQUEST["action"]) ? "request" : "no-request";
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            request_body: Some("{\"action\":\"save\"}".to_string()),
            request_method: Some("POST".to_string()),
            content_type: Some("application/json".to_string()),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(execution.stdout, "{\"action\":\"save\"}|no-post|no-request");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_superglobals_parse_bracketed_names_and_duplicate_keys() {
    let program = parse(
        r#"<?php
echo $_GET["filter"]["post_status"];
echo "|";
echo $_GET["ids"][0];
echo ",";
echo $_GET["ids"][1];
echo "|";
echo $_GET["rows"][0]["id"];
echo ":";
echo $_GET["rows"][1]["id"];
echo "|";
echo $_GET["dup"];
echo "|";
echo $_POST["meta"]["_wpnonce"];
echo "|";
echo $_POST["ids"][2];
echo ",";
echo $_POST["ids"][3];
echo "|";
echo $_REQUEST["dup"];
echo "|";
echo $_REQUEST["submit"];
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            query_string: Some(
                "filter[post_status]=publish&ids[]=10&ids[]=11&rows[][id]=a&rows[][id]=b&dup=old&dup=new"
                    .to_string(),
            ),
            request_body: Some(
                "meta[_wpnonce]=token%2Bplus&ids[2]=20&ids[]=21&dup=post&submit=Save+Draft"
                    .to_string(),
            ),
            request_method: Some("POST".to_string()),
            content_type: Some("application/x-www-form-urlencoded".to_string()),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "publish|10,11|a:b|new|token+plus|20,21|post|Save Draft"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_superglobals_normalize_dotted_and_spaced_root_names() {
    let program = parse(
        r#"<?php
echo $_GET["user_login"];
echo "|";
echo $_GET["remember_me"];
echo "|";
echo $_GET["nested_key"]["child space"];
echo "|";
echo $_POST["action_name"];
echo "|";
echo $_POST["form_name"]["inner.dot"];
echo "|";
echo $_REQUEST["dup_key"];
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            query_string: Some(
                "user.login=admin&remember+me=1&nested.key[child space]=query&dup.key=get"
                    .to_string(),
            ),
            request_body: Some(
                "action.name=save&form+name[inner.dot]=body&dup.key=post".to_string(),
            ),
            request_method: Some("POST".to_string()),
            content_type: Some("application/x-www-form-urlencoded".to_string()),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(execution.stdout, "admin|1|query|save|body|post");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_nested_reference_slots_survive_literal_path_array_copies() {
    let execution = run_source(
        r#"<?php
function handle_request_payload() {
    $_REQUEST["payload"] = ["a" => "one"];
    $alias =& $_REQUEST["payload"]["a"];
    $copy = $_REQUEST["payload"];
    $alias = "two";
    echo $_REQUEST["payload"]["a"], "|", $copy["a"], "|";
    $copy["a"] = "three";
    echo $alias, "|", $_REQUEST["payload"]["a"];
}

handle_request_payload();
echo "\n";

$items = ["outer" => ["slot" => "alpha"]];
$slot =& $items["outer"]["slot"];
$outer = $items["outer"];
$slot = "beta";
echo $items["outer"]["slot"], "|", $outer["slot"], "|";
$outer["slot"] = "gamma";
echo $slot, "|", $items["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "two|two|three|three\nbeta|beta|gamma|gamma"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_bag_reference_parameter_unset_detaches_before_later_local_writes() {
    let execution = run_source(
        r#"<?php
function normalize(&$slot) {
    $slot = "mutated";
    unset($slot);
    $slot = "local-only";
}

$_REQUEST["payload"] = ["slot" => "seed"];
normalize($_REQUEST["payload"]["slot"]);
echo $_REQUEST["payload"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "mutated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_input_env_cli_snapshot_matches_current_sapi_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = "tests/fixtures/milestone1278/request_input_seed.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PHPC_QUERY_STRING", "preview=true&name=WordPress+Core")
        .env("PHPC_REQUEST_METHOD", "POST")
        .env("PHPC_CONTENT_TYPE", "application/x-www-form-urlencoded")
        .env("PHPC_REQUEST_BODY", "action=save&space=hello+world")
        .args(["run", fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture}: {error}"));

    let actual = render_cli_snapshot(&output);
    let expected = "exit: 0\nstdout:\nPOST|preview=true&name=WordPress+Core|true|WordPress Core|save|hello world|true|save|action=save&space=hello+world--- stdout end ---\nstderr:\n--- stderr end ---\n";

    assert_eq!(actual, expected);
}

#[test]
fn bracketed_request_input_env_cli_snapshot_matches_current_sapi_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = "tests/fixtures/milestone1283/bracketed_request_input.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env(
            "PHPC_QUERY_STRING",
            "filter[post_status]=publish&ids[]=10&ids[]=11&dup=old&dup=new",
        )
        .env("PHPC_REQUEST_METHOD", "POST")
        .env("PHPC_CONTENT_TYPE", "application/x-www-form-urlencoded")
        .env(
            "PHPC_REQUEST_BODY",
            "meta[_wpnonce]=token%2Bplus&ids[2]=20&ids[]=21&dup=post&submit=Save+Draft",
        )
        .args(["run", fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture}: {error}"));

    let actual = render_cli_snapshot(&output);
    let expected = "exit: 0\nstdout:\npublish|10,11|empty|new|token+plus|20,21|post|Save Draft--- stdout end ---\nstderr:\n--- stderr end ---\n";

    assert_eq!(actual, expected);
}

#[test]
fn normalized_request_input_env_cli_snapshot_matches_current_sapi_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = "tests/fixtures/milestone1288/normalized_request_input.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env(
            "PHPC_QUERY_STRING",
            "user.login=admin&remember+me=1&nested.key[child space]=query&dup.key=get",
        )
        .env("PHPC_REQUEST_METHOD", "POST")
        .env("PHPC_CONTENT_TYPE", "application/x-www-form-urlencoded")
        .env(
            "PHPC_REQUEST_BODY",
            "action.name=save&form+name[inner.dot]=body&dup.key=post",
        )
        .args(["run", fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture}: {error}"));

    let actual = render_cli_snapshot(&output);
    let expected = "exit: 0\nstdout:\nadmin|1|query|save|body|post--- stdout end ---\nstderr:\n--- stderr end ---\n";

    assert_eq!(actual, expected);
}

#[test]
fn cookie_request_input_env_cli_snapshot_matches_current_sapi_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = "tests/fixtures/milestone1293/cookie_request_seed.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env(
            "PHPC_COOKIE",
            "wordpress_test_cookie=WP+Cookie+check; logged_in=user%7Ctoken; settings[theme]=classic; dotted.name=value",
        )
        .env("PHPC_QUERY_STRING", "preview=true")
        .args(["run", fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture}: {error}"));

    let actual = render_cli_snapshot(&output);
    let expected = "exit: 0\nstdout:\nWP Cookie check|user|token|classic|value|true|request-empty|wordpress_test_cookie=WP+Cookie+check; logged_in=user%7Ctoken; settings[theme]=classic; dotted.name=value--- stdout end ---\nstderr:\n--- stderr end ---\n";

    assert_eq!(actual, expected);
}

#[test]
fn files_superglobal_is_materialized_as_empty_auto_global_array() {
    let execution = run_source(
        r#"<?php
echo is_array($_FILES) ? "files-array" : "files-missing";
echo "|";
echo isset($_FILES["async-upload"]) ? "upload" : "files-empty";

function seed_upload_file() {
    $_FILES["async-upload"] = ["name" => "plugin.zip", "error" => 0];
}

seed_upload_file();
echo "|";
echo $_FILES["async-upload"]["name"];
echo ":";
echo $_FILES["async-upload"]["error"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "files-array|files-empty|plugin.zip:0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn files_superglobal_parses_explicit_cli_upload_metadata_seed() {
    let program = parse(
        r#"<?php
echo $_FILES["async-upload"]["name"];
echo "|";
echo $_FILES["async-upload"]["type"];
echo "|";
echo $_FILES["async-upload"]["tmp_name"];
echo "|";
echo $_FILES["async-upload"]["error"];
echo "|";
echo $_FILES["async-upload"]["size"];
echo "|";
echo $_FILES["async-upload"]["full_path"];
echo "|";

function upload_summary() {
    return $_FILES["async-upload"]["name"] . ":" . $_FILES["async-upload"]["size"];
}

echo upload_summary();
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            upload_files: Some(
                "async-upload[name]=plugin.zip&async-upload[type]=application%2Fzip&async-upload[tmp_name]=%2Ftmp%2Fphpc-upload&async-upload[error]=0&async-upload[size]=12345&async-upload[full_path]=plugin.zip"
                    .to_string(),
            ),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "plugin.zip|application/zip|/tmp/phpc-upload|0|12345|plugin.zip|plugin.zip:12345"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn upload_files_env_cli_snapshot_matches_current_sapi_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = "tests/fixtures/milestone1298/upload_files_metadata_seed.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env(
            "PHPC_FILES",
            "async-upload[name]=plugin.zip&async-upload[type]=application%2Fzip&async-upload[tmp_name]=%2Ftmp%2Fphpc-upload&async-upload[error]=0&async-upload[size]=12345&async-upload[full_path]=plugin.zip",
        )
        .args(["run", fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture}: {error}"));

    let actual = render_cli_snapshot(&output);
    let expected = "exit: 0\nstdout:\nplugin.zip|application/zip|/tmp/phpc-upload|0|12345|plugin.zip|plugin.zip:12345--- stdout end ---\nstderr:\n--- stderr end ---\n";

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}

#[test]
fn emit_ir_rejects_request_bag_superglobals_until_native_request_state_exists() {
    let direct = emit_ir_source("<?php\necho $_GET;\n").unwrap_err();
    assert_eq!(direct.phase, Phase::Codegen);
    assert_eq!(direct.line, 2);
    assert_eq!(direct.column, 6);
    assert_eq!(direct.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION);

    let isset_offset = emit_ir_source("<?php\necho isset($_REQUEST['preview']);\n").unwrap_err();
    assert_eq!(isset_offset.phase, Phase::Codegen);
    assert_eq!(isset_offset.line, 2);
    assert_eq!(isset_offset.column, 12);
    assert_eq!(isset_offset.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION);

    let files_direct = emit_ir_source("<?php\necho $_FILES;\n").unwrap_err();
    assert_eq!(files_direct.phase, Phase::Codegen);
    assert_eq!(files_direct.line, 2);
    assert_eq!(files_direct.column, 6);
    assert_eq!(files_direct.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION);

    let cookie_direct = emit_ir_source("<?php\necho $_COOKIE;\n").unwrap_err();
    assert_eq!(cookie_direct.phase, Phase::Codegen);
    assert_eq!(cookie_direct.line, 2);
    assert_eq!(cookie_direct.column, 6);
    assert_eq!(cookie_direct.message, LLVM_REQUEST_SUPERGLOBAL_REJECTION);
}

#[test]
fn emit_ir_request_state_ordinary_array_key_consumers_share_blocker() {
    for (source, subject) in [
        (
            "<?php\necho $local[$_GET[\"preview\"]];\n",
            "$_GET[\"preview\"]",
        ),
        (
            "<?php\n$local[$_POST[\"action\"]] = \"x\";\n",
            "$_POST[\"action\"]",
        ),
        (
            "<?php\nunset($local[$_SERVER[\"SCRIPT_NAME\"]]);\n",
            "$_SERVER[\"SCRIPT_NAME\"]",
        ),
        (
            "<?php\n$alias =& $local[$_COOKIE[\"wordpress_test_cookie\"]];\n",
            "$_COOKIE[\"wordpress_test_cookie\"]",
        ),
        (
            "<?php\nfor ($local[$_REQUEST[\"name\"]] = 0; false; ) {}\n",
            "$_REQUEST[\"name\"]",
        ),
        (
            "<?php\n$local[$_GET[\"count\"]] .= \"x\";\n",
            "$_GET[\"count\"]",
        ),
        (
            "<?php\necho ($local[$_FILES[\"upload\"]] ??= \"x\");\n",
            "$_FILES[\"upload\"]",
        ),
        (
            "<?php\necho ++$local[$_SESSION[\"id\"]];\n",
            "$_SESSION[\"id\"]",
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen, "{source}");
        assert_eq!(
            error.message,
            llvm_request_superglobal_array_key_consumer_rejection(subject),
            "{source}"
        );
    }
}

#[test]
fn native_request_state_handle_boundary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1639/native_request_state_handle_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = std::fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1639/native_request_state_handle_boundary_emit_ir.cli"),
    )
    .expect("native request-state CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn globals_direct_string_offsets_route_function_scope_writes_to_root() {
    let execution = run_source(
        r#"<?php
function boot_cache() {
    $GLOBALS["wp_object_cache"] = "ready";
}

function use_cache() {
    global $wp_object_cache;
    echo $wp_object_cache;
}

boot_cache();
use_cache();
echo "|", $GLOBALS["wp_object_cache"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ready|ready");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_direct_string_offsets_bind_reference_targets_to_direct_sources() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["target"] =& $value;
echo $target, "|", $GLOBALS["target"], "|";
$value = "second";
echo $target, "|", $GLOBALS["target"], "|";
$GLOBALS["target"] = "third";
echo $value, "|", $target;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first|second|second|third|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_reference_targets_can_bind_function_local_sources_to_root() {
    let execution = run_source(
        r#"<?php
function bind_target() {
    $value = "local";
    $GLOBALS["target"] =& $value;
    $value = "changed";
    echo $GLOBALS["target"], "|";
}

bind_target();
echo $target, "|";
$target = "global-write";
echo $GLOBALS["target"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed|global-write");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_reference_targets_detach_when_source_name_is_unset() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["target"] =& $value;
unset($value);
$value = "new";
echo $GLOBALS["target"], "|", $value, "|";
$GLOBALS["target"] = "global";
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|new|new");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_reference_targets_bind_direct_sources_to_root_arrays() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["bag"]["slot"] =& $value;
echo $bag["slot"], "|", $GLOBALS["bag"]["slot"], "|";
$value = "second";
echo $bag["slot"], "|";
$GLOBALS["bag"]["slot"] = "third";
echo $value, "|", $bag["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first|second|third|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_reference_targets_can_bind_function_local_sources() {
    let execution = run_source(
        r#"<?php
function bind_nested() {
    $value = "local";
    $GLOBALS["bag"]["slot"] =& $value;
    $value = "changed";
    echo $GLOBALS["bag"]["slot"], "|";
}

bind_nested();
echo $bag["slot"], "|";
$bag["slot"] = "global-write";
echo $GLOBALS["bag"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed|global-write");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_append_reference_targets_bind_selected_auto_key() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["bag"][] =& $value;
echo $bag[0], "|";
$value = "second";
echo $GLOBALS["bag"][0], "|";
$bag[0] = "third";
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|second|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_append_reference_sources_bind_alias_to_root_array_slot() {
    let execution = run_source(
        r#"<?php
$GLOBALS["bag"] = [];
$alias =& $GLOBALS["bag"][];
$alias = "from-alias";
echo $bag[0], "|";
$GLOBALS["bag"][0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_append_reference_sources_bind_function_local_alias() {
    let execution = run_source(
        r#"<?php
function bind_nested() {
    $alias =& $GLOBALS["bag"]["outer"][];
    $alias = "from-alias";
    echo $GLOBALS["bag"]["outer"][0], "|";
    $GLOBALS["bag"]["outer"][0] = "from-slot";
    echo $alias;
}

bind_nested();
echo "|", $bag["outer"][0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn other_superglobals_remain_ordinary_missing_variables_for_now() {
    let error = runtime_error("<?php\necho $_SESSION;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$_SESSION'");
}
