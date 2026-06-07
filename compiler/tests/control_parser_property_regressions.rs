use php_compiler::{run_source, run_source_with_source_file};

#[test]
fn parser_accepts_keyword_class_constant_names_and_fetches() {
    let execution = run_source(
        r#"<?php
class Obj {
    const DECLARE = 'declare',
          RETURN = 'return',
          FUNCTION = 'function',
          USE = 'use';
    const TRAIT = 'trait';
    const STATIC = 'static';
    const ABSTRACT = 'abstract';
    const FINAL = 'final';
    const PUBLIC = 'public';
    const PROTECTED = 'protected';
    const PRIVATE = 'private';
}

echo Obj::DECLARE, "\n";
echo Obj::RETURN, "\n";
echo Obj::FUNCTION, "\n";
echo Obj::USE, "\n";
echo Obj::
    USE, "\n";
echo Obj::TRAIT, "\n";
echo Obj::STATIC, "\n";
echo Obj::ABSTRACT, "\n";
echo Obj::FINAL, "\n";
echo Obj::PUBLIC, "\n";
echo Obj::PROTECTED, "\n";
echo Obj::PRIVATE, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "declare\nreturn\nfunction\nuse\nuse\ntrait\nstatic\nabstract\nfinal\npublic\nprotected\nprivate\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn compound_assignment_to_unset_untyped_declared_property_reinitializes() {
    let execution = run_source(
        r#"<?php
class C {
    public $a;
    function errorHandler($errno, $errstr) {
        unset($this->a);
    }
}
$c = new C;
set_error_handler([$c, 'errorHandler']);
unset($c->a);
$c->a += 5;
var_dump($c->a);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(5)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn increment_decrement_unset_untyped_declared_property_uses_null_initial_value() {
    let execution = run_source(
        r#"<?php
class C {
    public $a;
    function errorHandler($errno, $errstr) {
        unset($this->a);
    }
}
$c = new C;
set_error_handler([$c, 'errorHandler']);
unset($c->a);
++$c->a;
var_dump($c->a);
unset($c->a);
--$c->a;
var_dump($c->a);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(1)\nNULL\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fatal_output_after_var_dump_is_preserved_in_cli_bytes() {
    let execution = run_source_with_source_file(
        r#"<?php
function foo($a) {
   try {
     throw new Exception("ex");
   } finally {
     var_dump($a);
   }
}

foo("finally");
"#,
        "tests/fixtures/control_parser_property.php",
    )
    .unwrap();

    let stdout_bytes = String::from_utf8(execution.stdout_bytes.clone()).unwrap();
    assert_eq!(stdout_bytes, execution.stdout);
    assert!(execution.stdout.contains("string(7) \"finally\"\n\nFatal error: Uncaught Exception: ex in tests/fixtures/control_parser_property.php:4"));
    assert!(execution.stdout.contains("Stack trace:\n#0 tests/fixtures/control_parser_property.php(10): foo('finally')\n#1 {main}"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}
