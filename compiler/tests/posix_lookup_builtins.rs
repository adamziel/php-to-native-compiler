#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source, run_source_with_source_file};

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
fn posix_access_terminal_process_and_times_metadata_cover_current_rows() {
    let source = r#"<?php
foreach ([-1, 01000, 02000] as $flags) {
    try {
        posix_access(__FILE__, $flags);
    } catch (ValueError $e) {
        echo "flag;";
    }
}
echo posix_access(__FILE__, POSIX_F_OK) ? "exists;" : "missing;";
echo posix_access(__FILE__, POSIX_R_OK | POSIX_W_OK) ? "rw;" : "no-rw;";
var_dump(posix_isatty(STDIN));
var_dump(posix_ttyname(STDIN));
posix_kill((2 ** 22) + 1, 9);
echo "errno=", posix_errno(), ";";
try {
    posix_kill(PHP_INT_MAX, SIGTERM);
} catch (ValueError $e) {
    echo "kill-range;";
}
try {
    posix_setpgid(-2, 1);
} catch (ValueError $e) {
    echo "setpid;";
}
try {
    posix_setpgid(1, -2);
} catch (ValueError $e) {
    echo "setpgid;";
}
var_dump(posix_getpgid(-99));
$nproc = posix_sysconf(POSIX_SC_NPROCESSORS_ONLN);
$open = posix_sysconf(POSIX_SC_OPEN_MAX);
echo is_int($nproc) && $nproc > 0 ? "nproc;" : "bad-nproc;";
echo is_int($open) && $open >= 256 ? "open;" : "bad-open;";
$times = posix_times();
echo is_array($times)
    && is_int($times["ticks"])
    && is_int($times["utime"])
    && is_int($times["stime"])
    && is_int($times["cutime"])
    && is_int($times["cstime"])
    ? "times"
    : "bad-times";
"#;
    let path = temp_source_path("posix-terminal-process");
    fs::write(&path, source).expect("temporary POSIX terminal/process source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "flag;flag;flag;exists;rw;bool(false)\nbool(false)\nerrno=3;kill-range;setpid;setpgid;bool(false)\nnproc;open;times"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_fd_helpers_emit_php_shaped_weak_argument_warnings() {
    let execution = run_source(
        r#"<?php
class PosixStringable {
    public function __toString() {
        return "1";
    }
}
foreach ([null, 5.5, "5.5", "Hello", [], new PosixStringable()] as $value) {
    var_dump(posix_isatty($value));
}
var_dump(posix_ttyname(new PosixStringable()));
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: posix_isatty(): Passing null to parameter #1 ($file_descriptor) of type int is deprecated"
    ));
    assert!(execution
        .stdout
        .contains("Deprecated: Implicit conversion from float 5.5 to int loses precision"));
    assert!(execution.stdout.contains(
        "Deprecated: Implicit conversion from float-string \"5.5\" to int loses precision"
    ));
    assert!(execution.stdout.contains(
        "Warning: posix_isatty(): Argument #1 ($file_descriptor) must be of type int|resource, string given"
    ));
    assert!(execution.stdout.contains(
        "Warning: posix_isatty(): Argument #1 ($file_descriptor) must be of type int|resource, array given"
    ));
    assert!(execution.stdout.contains(
        "Warning: posix_isatty(): Argument #1 ($file_descriptor) must be of type int|resource, PosixStringable given"
    ));
    assert!(execution.stdout.contains(
        "Warning: posix_ttyname(): Argument #1 ($file_descriptor) must be of type int|resource, PosixStringable given"
    ));
    assert!(execution
        .stdout
        .contains("Warning: Object of class PosixStringable could not be converted to int"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_identity_mutation_builtins_cover_current_identity_rows() {
    let source = r#"<?php
$gid = posix_getgid();
$egid = posix_getegid();
$uid = posix_getuid();
$euid = posix_geteuid();
$bad_gid = $gid === 0 ? 1 : 0;
$bad_egid = $egid === 0 ? 1 : 0;
$bad_uid = $uid === 0 ? 1 : 0;
$bad_euid = $euid === 0 ? 1 : 0;
echo function_exists("posix_setgid")
    && function_exists("posix_setuid")
    && is_callable("posix_setegid")
    && is_callable("posix_errno")
    && is_callable("posix_strerror")
    ? "known"
    : "missing";
echo "|";
var_dump(posix_setgid($gid));
var_dump(posix_setgid($bad_gid));
var_dump(posix_setgid(-2345));
var_dump(posix_setegid($egid));
var_dump(posix_setegid($bad_egid));
var_dump(posix_seteuid($euid));
var_dump(posix_seteuid($bad_euid));
var_dump(posix_seteuid(-12345));
var_dump(posix_setuid($uid));
var_dump(posix_errno());
var_dump(posix_setuid($bad_uid));
var_dump(posix_errno());
var_dump(posix_setuid(-12345));
var_dump(posix_errno());
echo gettype(posix_strerror(posix_errno())), "|";
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
        "known|bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nint(0)\nbool(false)\nint(1)\nbool(false)\nint(22)\nstring|1/1/user_id/posix/return"
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
echo function_exists("posix_setuid") ? "1" : "0";
echo is_callable("posix_seteuid") ? "1" : "0";
echo function_exists("posix_setegid") ? "1" : "0";
echo function_exists("posix_access") ? "1" : "0";
echo is_callable("posix_getpgid") ? "1" : "0";
echo function_exists("posix_getsid") ? "1" : "0";
echo is_callable("posix_uname") ? "1" : "0";
echo function_exists("posix_errno") ? "1" : "0";
echo is_callable("posix_strerror") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 16, "{ir}");
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
