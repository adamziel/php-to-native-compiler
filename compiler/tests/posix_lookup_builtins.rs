#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn posix_lookup_builtins_return_local_account_and_group_arrays() {
    let source = r#"<?php
$pw = posix_getpwuid(getmyuid());
$gr = posix_getgrgid(getmygid());
echo function_exists("posix_getpwuid") && is_callable("posix_getgrgid") ? "known" : "missing";
echo "|";
echo is_array($pw)
    && is_string($pw["name"])
    && is_string($pw["passwd"])
    && is_int($pw["uid"])
    && is_int($pw["gid"])
    && array_key_exists("gecos", $pw)
    && array_key_exists("dir", $pw)
    && array_key_exists("shell", $pw)
    ? "pw-array"
    : "bad-pw";
echo "|";
echo is_array($gr)
    && is_string($gr["name"])
    && is_string($gr["passwd"])
    && is_array($gr["members"])
    && is_int($gr["gid"])
    ? "gr-array"
    : "bad-gr";
echo "|";
$rf = new ReflectionFunction("posix_getpwuid");
echo $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters(), "/";
echo $rf->getParameters()[0]->getName(), "/", $rf->getExtensionName(), "/";
echo $rf->hasReturnType() ? "return" : "no-return";
echo "|";
$viaInvoke = $rf->invoke(getmyuid());
echo is_array($viaInvoke) && is_int($viaInvoke["uid"]) ? "invoke-array" : "bad-invoke";
echo "|";
var_dump(posix_getpwuid(-99));
var_dump(posix_getgrgid(-999));
"#;
    let path = temp_source_path("posix-lookup");
    fs::write(&path, source).expect("temporary POSIX lookup source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "known|pw-array|gr-array|1/1/user_id/posix/return|invoke-array|bool(false)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_identity_and_name_lookup_builtins_cover_basic_rows() {
    let source = r#"<?php
$uid = posix_getuid();
$euid = posix_geteuid();
$gid = posix_getgid();
$egid = posix_getegid();
$pid = posix_getpid();
$ppid = posix_getppid();
$pgrp = posix_getpgrp();
$groups = posix_getgroups();
$pw = posix_getpwuid($uid);
$pw_by_name = is_array($pw) ? posix_getpwnam($pw["name"]) : false;
$gr = posix_getgrgid($gid);
$gr_by_name = is_array($gr) ? posix_getgrnam($gr["name"]) : false;
echo function_exists("posix_getuid") && is_callable("posix_getgroups") ? "known" : "missing";
echo "|";
echo is_int($uid) && $uid >= 0
    && is_int($euid) && $euid >= 0
    && is_int($gid) && $gid >= 0
    && is_int($egid) && $egid >= 0
    ? "ids"
    : "bad-ids";
echo "|";
echo is_int($pid) && $pid > 0
    && is_int($ppid) && $ppid >= 0
    && is_int($pgrp) && $pgrp >= 0
    ? "process"
    : "bad-process";
echo "|";
echo is_array($groups) ? "groups" : "bad-groups";
echo "|";
echo is_array($pw)
    && $pw["uid"] === $uid
    && is_array($pw_by_name)
    && $pw_by_name["uid"] === $uid
    ? "passwd"
    : "bad-passwd";
echo "|";
echo is_array($gr)
    && $gr["gid"] === $gid
    && is_array($gr_by_name)
    && $gr_by_name["gid"] === $gid
    ? "group"
    : "bad-group";
echo "|";
$rf = new ReflectionFunction("posix_getpwnam");
echo $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters(), "/";
echo $rf->getParameters()[0]->getName(), "/", $rf->getExtensionName(), "/";
echo $rf->hasReturnType() ? "return" : "no-return";
echo "|";
var_dump(posix_getpwnam(""));
var_dump(posix_getgrnam(""));
"#;
    let path = temp_source_path("posix-identity-lookup");
    fs::write(&path, source).expect("temporary POSIX identity source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "known|ids|process|groups|passwd|group|1/1/username/posix/return|bool(false)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_access_process_metadata_and_uname_cover_basic_rows() {
    let source = r#"<?php
$pid = posix_getpid();
$pgid = posix_getpgid($pid);
$sid = posix_getsid($pid);
$uname = posix_uname();
echo function_exists("posix_access") && is_callable("posix_uname") ? "known" : "missing";
echo "|";
var_dump(posix_access(str_repeat("bogus path", 1042)));
echo "|";
echo is_int($pgid) && $pgid >= 0 ? "pgid" : "bad-pgid";
echo "|";
echo is_int($sid) && $sid >= 0 ? "sid" : "bad-sid";
echo "|";
echo is_array($uname)
    && is_string($uname["sysname"])
    && is_string($uname["nodename"])
    && is_string($uname["release"])
    && is_string($uname["version"])
    && is_string($uname["machine"])
    ? "uname"
    : "bad-uname";
echo "|";
$rf = new ReflectionFunction("posix_getsid");
echo $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters(), "/";
echo $rf->getParameters()[0]->getName(), "/", $rf->getExtensionName(), "/";
echo $rf->hasReturnType() ? "return" : "no-return";
echo "|";
try {
    posix_getsid(-1);
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage();
}
"#;
    let path = temp_source_path("posix-access-process");
    fs::write(&path, source).expect("temporary POSIX access/process source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "known|bool(false)\n|pgid|sid|uname|1/1/process_id/posix/return|ValueError: posix_getsid(): Argument #1 ($process_id) must be between 0 and 2147483647"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_identity_mutation_builtins_cover_current_identity_rows() {
    let source = r#"<?php
$gid = posix_getgid();
$egid = posix_getegid();
$euid = posix_geteuid();
$bad_gid = $gid === 0 ? 1 : 0;
$bad_egid = $egid === 0 ? 1 : 0;
$bad_euid = $euid === 0 ? 1 : 0;
echo function_exists("posix_setgid") && is_callable("posix_setegid") ? "known" : "missing";
echo "|";
var_dump(posix_setgid($gid));
var_dump(posix_setgid($bad_gid));
var_dump(posix_setgid(-2345));
var_dump(posix_setegid($egid));
var_dump(posix_setegid($bad_egid));
var_dump(posix_seteuid($euid));
var_dump(posix_seteuid($bad_euid));
var_dump(posix_seteuid(-12345));
$rf = new ReflectionFunction("posix_seteuid");
echo $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters(), "/";
echo $rf->getParameters()[0]->getName(), "/", $rf->getExtensionName(), "/";
echo $rf->hasReturnType() ? "return" : "no-return";
"#;
    let path = temp_source_path("posix-identity-mutation");
    fs::write(&path, source).expect("temporary POSIX identity mutation source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "known|bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\n1/1/user_id/posix/return"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_posix_lookup_names_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("posix_getpwuid") ? "1" : "0";
echo is_callable("posix_getgrgid") ? "1" : "0";
echo function_exists("posix_getuid") ? "1" : "0";
echo is_callable("posix_getgroups") ? "1" : "0";
echo function_exists("posix_getpwnam") ? "1" : "0";
echo is_callable("posix_getgrnam") ? "1" : "0";
echo function_exists("posix_setgid") ? "1" : "0";
echo is_callable("posix_seteuid") ? "1" : "0";
echo function_exists("posix_setegid") ? "1" : "0";
echo function_exists("posix_access") ? "1" : "0";
echo is_callable("posix_getpgid") ? "1" : "0";
echo function_exists("posix_getsid") ? "1" : "0";
echo is_callable("posix_uname") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 13, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho posix_getpwuid(0);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

fn temp_source_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "phpc-{label}-{}-{}.php",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ))
}
