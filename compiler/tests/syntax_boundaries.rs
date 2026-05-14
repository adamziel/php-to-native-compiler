use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_INTERFACE_REJECTION: &str = "LLVM interface lowering rejects interface declarations until native class/interface tables, implementation checks, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current interface metadata behavior";
const LLVM_TRAIT_REJECTION: &str = "LLVM trait lowering rejects trait declarations until native trait tables, class trait-use composition, conflict resolution, aliasing, relationship metadata, autoload interaction, and exact native error behavior exist; phpc run handles current trait metadata behavior";
const LLVM_ENUM_REJECTION: &str = "LLVM enum lowering rejects enum declarations until native class/enum tables, enum case objects, backed enum values, interface implementation, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current enum metadata behavior";

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn lex_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Lex);
    error
}

#[test]
fn long_array_literals_execute_as_short_array_aliases() {
    let execution = run_source(
        r#"<?php
$items = array(
    "first",
    2 => "two",
    "2" => "two updated",
    "02" => "zero two",
    "name" => "Ada",
    1 + 2 => "three",
);
$upper = ARRAY("a", "b");
echo count($items), "\n";
echo $items[0], "|", $items[2], "|", $items["02"], "|", $items["name"], "|", $items[3], "\n";
echo $upper[1], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5\nfirst|two updated|zero two|Ada|three\nb\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unsupported_heredoc_nowdoc_syntax_has_stable_lex_errors() {
    let cases = [
        ("<?php\n$text = <<<TXT\nhello\nTXT;\n", 2, 9),
        ("<?php\n$text = <<<'TXT'\nhello\nTXT;\n", 2, 9),
    ];

    for (source, line, column) in cases {
        let error = lex_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported heredoc/nowdoc string syntax: multiline string literals are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_heredoc_nowdoc_syntax_at_lex_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$text = <<<TXT\nhello\nTXT;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Lex);
    assert_eq!(
        error.message,
        "unsupported heredoc/nowdoc string syntax: multiline string literals are not implemented"
    );
}

#[test]
fn unsupported_array_item_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$values = [1, 2];
$items = array(...$values);
"#,
            3,
            16,
            "unsupported array spread: spread elements are not implemented",
        ),
        (
            r#"<?php
$value = "Ada";
$items = array(&$value);
"#,
            3,
            16,
            "unsupported array reference element: references are not implemented",
        ),
        (
            r#"<?php
$values = [1, 2];
$items = [...$values];
"#,
            3,
            11,
            "unsupported array spread: spread elements are not implemented",
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
fn unsupported_array_destructuring_assignments_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n[$first] = [1];\n",
            2,
            10,
            "unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\necho list($first);\n",
            2,
            6,
            "unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\nlist($first[0]) = [1];\n",
            2,
            12,
            "unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\nlist(, $second) = [1, 2];\n",
            2,
            6,
            "unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented",
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
fn emit_ir_rejects_array_destructuring_assignment_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n[$first] = [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented"
    );
}

#[test]
fn unsupported_unset_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = [[1]];
unset($items[0][0]);
"#,
            3,
            16,
        ),
        (
            r#"<?php
$items = [];
UNSET($items[]);
"#,
            3,
            13,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported unset: only direct variables like unset($name), direct array offset removal like unset($array[$key]), and direct static property operands like unset(ClassName::$property) are implemented; object property, append, and nested unset forms are not implemented"
        );
    }
}

#[test]
fn object_property_unset_has_stable_parse_boundary() {
    let error = parse_error(
        r#"<?php
class Box {
    public $name;
}
$box = new Box();
unset($box->name);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled"
    );
}

#[test]
fn emit_ir_rejects_object_property_unset_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Box { public $name; }\n$box = new Box();\nunset($box->name);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled"
    );
}

#[test]
fn unsupported_exception_syntax_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$value = throw new Exception('boom');\n",
            2,
            10,
            "unsupported throw: exception objects and stack unwinding are not implemented",
        ),
        (
            "<?php\nCATCH (Exception $e) {\n    echo 'caught';\n}\n",
            2,
            1,
            "unexpected catch: catch must follow a try block",
        ),
        (
            "<?php\nFINALLY {\n    echo 'done';\n}\n",
            2,
            1,
            "unexpected finally: finally must follow a try block",
        ),
        (
            "<?php\ntry {\n    echo 'work';\n}\n",
            2,
            1,
            "expected catch or finally after try block",
        ),
        ("<?php\ntry echo 'work';\n", 2, 5, "expected try block"),
        (
            "<?php\ntry {\n} catch () {\n}\n",
            3,
            10,
            "expected catch type name",
        ),
        (
            "<?php\ntry {\n} catch (Exception| $e) {\n}\n",
            3,
            21,
            "expected catch type name",
        ),
        (
            "<?php\ntry {\n} catch (Exception $e)\n",
            4,
            1,
            "expected catch block",
        ),
        ("<?php\ntry {\n} finally\n", 4, 1, "expected finally block"),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn emit_ir_rejects_throw_statement_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\nthrow new Exception('boom');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary"
    );
}

#[test]
fn unsupported_yield_syntax_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\nyield $value;\n",
            2,
            1,
            "unsupported yield expression: generators and generator object execution are not implemented",
        ),
        (
            "<?php\nYIELD from [1, 2];\n",
            2,
            1,
            "unsupported yield expression: generators and generator object execution are not implemented",
        ),
        (
            "<?php\necho yield 1;\n",
            2,
            6,
            "unsupported yield expression: generators and generator object execution are not implemented",
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
fn emit_ir_rejects_yield_syntax_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\nyield from [1, 2];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported yield expression: generators and generator object execution are not implemented"
    );
}

#[test]
fn unsupported_match_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$value = match ($status) {\n    200 => 'ok',\n    default => 'other',\n};\n",
            2,
            10,
        ),
        (
            "<?php\nMATCH ($status) {\n    200 => 'ok',\n    default => 'other',\n};\n",
            2,
            1,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported match expression: expression-form branching is not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_match_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\n$value = match ($status) {\n    default => 'other',\n};\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported match expression: expression-form branching is not implemented"
    );
}

#[test]
fn unsupported_exponentiation_syntax_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\necho 2 ** 3;\n",
            2,
            8,
            "unsupported exponentiation operator: ** and **= are not implemented",
        ),
        (
            "<?php\n$value = 2;\n$value **= 3;\n",
            3,
            8,
            "unsupported exponentiation operator: ** and **= are not implemented",
        ),
        (
            "<?php\n$value = 2;\necho ($value **= 3);\n",
            3,
            14,
            "unsupported exponentiation operator: ** and **= are not implemented",
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
fn emit_ir_rejects_exponentiation_syntax_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\necho 2 ** 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported exponentiation operator: ** and **= are not implemented"
    );
}

#[test]
fn unsupported_first_class_callable_syntax_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$callback = strlen(...);\n",
            2,
            20,
            "unsupported first-class callable syntax: Closure creation with ... is not implemented",
        ),
        (
            "<?php\n$callback = 'strlen';\necho $callback(...);\n",
            3,
            16,
            "unsupported first-class callable syntax: Closure creation with ... is not implemented",
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
fn emit_ir_rejects_first_class_callable_syntax_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$callback = strlen(...);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported first-class callable syntax: Closure creation with ... is not implemented"
    );
}

#[test]
fn unsupported_magic_class_name_instantiation_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\necho new self();\n",
            2,
            10,
            "unsupported magic class name: self, parent, and static class name resolution is not implemented",
        ),
        (
            "<?php\necho new parent();\n",
            2,
            10,
            "unsupported magic class name: self, parent, and static class name resolution is not implemented",
        ),
        (
            "<?php\necho new static();\n",
            2,
            10,
            "unsupported magic class name: self, parent, and static class name resolution is not implemented",
        ),
        (
            "<?php\necho new SELF();\n",
            2,
            10,
            "unsupported magic class name: self, parent, and static class name resolution is not implemented",
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
fn emit_ir_rejects_magic_class_name_instantiation_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\necho new self();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported magic class name: self, parent, and static class name resolution is not implemented"
    );
}

#[test]
fn unsupported_anonymous_class_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$box = new class {};\n",
            2,
            12,
            "unsupported anonymous class: anonymous classes are not implemented",
        ),
        (
            "<?php\n$box = new class() {};\n",
            2,
            12,
            "unsupported anonymous class: anonymous classes are not implemented",
        ),
        (
            "<?php\necho new class {};\n",
            2,
            10,
            "unsupported anonymous class: anonymous classes are not implemented",
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
fn emit_ir_rejects_anonymous_class_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$box = new class {};\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported anonymous class: anonymous classes are not implemented"
    );
}

#[test]
fn unsupported_clone_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$copy = clone $object;\n",
            2,
            9,
            "unsupported clone expression: object handle copying and __clone dispatch are not implemented",
        ),
        (
            "<?php\necho clone $object;\n",
            2,
            6,
            "unsupported clone expression: object handle copying and __clone dispatch are not implemented",
        ),
        (
            "<?php\nCLONE $object;\n",
            2,
            1,
            "unsupported clone expression: object handle copying and __clone dispatch are not implemented",
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
fn emit_ir_rejects_clone_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$copy = clone $object;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported clone expression: object handle copying and __clone dispatch are not implemented"
    );
}

#[test]
fn unsupported_instanceof_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$is = $object instanceof $class;\n",
            2,
            26,
            "unsupported instanceof class expression: dynamic class names are not implemented",
        ),
        (
            "<?php\n$is = $object INSTANCEOF $class;\n",
            2,
            26,
            "unsupported instanceof class expression: dynamic class names are not implemented",
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
fn emit_ir_rejects_instanceof_expression_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$is = $object instanceof Widget;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn unsupported_interface_declaration_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\ninterface Renderable extends Displayable {}\n",
            2,
            22,
            "unsupported interface inheritance: interface extends clauses are not implemented",
        ),
        (
            "<?php\ninterface Renderable {\n    const NAME = \"view\";\n}\n",
            3,
            5,
            "unsupported interface constant declaration: interface constants are not implemented",
        ),
        (
            "<?php\ninterface Renderable {\n    protected function render();\n}\n",
            3,
            5,
            "unsupported interface method declaration: only public interface methods are implemented",
        ),
        (
            "<?php\nif (true) {\n    interface Nested {}\n}\n",
            3,
            5,
            "unsupported interface declaration: only top-level interface declarations are implemented",
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
fn emit_ir_rejects_interface_declaration_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\ninterface Renderable {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INTERFACE_REJECTION);
}

#[test]
fn emit_asm_rejects_interface_declaration_before_backend_execution() {
    let error = php_compiler::emit_asm_source("<?php\ninterface Renderable {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INTERFACE_REJECTION);
}

#[test]
fn unsupported_trait_declaration_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\ntrait Reusable {\n    public function render() {}\n}\n",
            3,
            5,
            "unsupported trait member declaration: trait members and trait use execution are not implemented",
        ),
        (
            "<?php\nif (true) {\n    trait Nested {}\n}\n",
            3,
            5,
            "unsupported trait declaration: only top-level trait declarations are implemented",
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
fn emit_ir_rejects_trait_declaration_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\ntrait Reusable {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_TRAIT_REJECTION);
}

#[test]
fn emit_asm_rejects_trait_declaration_before_backend_execution() {
    let error = php_compiler::emit_asm_source("<?php\ntrait Reusable {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_TRAIT_REJECTION);
}

#[test]
fn unsupported_enum_declaration_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\nenum Status: string {\n    case Draft = \"draft\";\n}\n",
            2,
            12,
            "unsupported backed enum declaration: backed enum values and scalar backing types are not implemented",
        ),
        (
            "<?php\nenum Status implements Renderable {}\n",
            2,
            13,
            "unsupported enum interface implementation: enum implements clauses are not implemented",
        ),
        (
            "<?php\nenum Status {\n    public function label() {}\n}\n",
            3,
            5,
            "unsupported enum member declaration: only unbacked enum case declarations are implemented",
        ),
        (
            "<?php\nenum Status {\n    case Draft = \"draft\";\n}\n",
            3,
            16,
            "unsupported enum case value: backed enum case values are not implemented",
        ),
        (
            "<?php\nif (true) {\n    enum Nested {}\n}\n",
            3,
            5,
            "unsupported enum declaration: only top-level enum declarations are implemented",
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
fn emit_ir_rejects_enum_declaration_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\nenum Status { case Draft; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ENUM_REJECTION);
}

#[test]
fn emit_asm_rejects_enum_declaration_before_backend_execution() {
    let error = php_compiler::emit_asm_source("<?php\nenum Status { case Draft; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ENUM_REJECTION);
}

#[test]
fn goto_labels_execute_for_current_statement_list_subset() {
    let execution = run_source(
        r#"<?php
echo "a";
goto after;
echo "skipped";
after:
echo "b";
while (true) {
    if (true) {
        goto end_loop;
    }
    echo "never";
    end_loop:
    echo "c";
    break;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "abc");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unresolved_goto_label_has_stable_runtime_error() {
    let error = run_source("<?php\ngoto missing;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined goto label 'missing'");
}

#[test]
fn unsupported_goto_expression_has_stable_parse_error() {
    let error = parse_error("<?php\necho goto done;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported goto: goto statements and labels are not implemented"
    );
}

#[test]
fn emit_ir_rejects_goto_after_parse() {
    let error = php_compiler::emit_ir_source("<?php\ngoto done;\ndone:\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
}

#[test]
fn unparenthesized_nested_ternary_has_stable_parse_error() {
    let cases = [
        (
            "<?php\n$flag = true;\n$result = $flag ? false ? 'bad' : 'inner' : 'outer';\n",
            3,
            25,
            "unsupported nested ternary expression: parenthesize nested ternary expressions in the current subset",
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
fn emit_ir_rejects_ternary_expression_after_parse() {
    let error = php_compiler::emit_ir_source(
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? 0 : 5;\necho $maybe ? 1 : 2;\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior"
    );
}

#[test]
fn unsupported_chained_null_coalescing_has_stable_parse_error() {
    let error = parse_error("<?php\n$first = null;\n$result = $first ?? $second ?? 'fallback';\n");
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 29);
    assert_eq!(
        error.message,
        "unsupported null coalescing expression: null-aware expression-form branching is not implemented"
    );
}

#[test]
fn unsupported_null_coalescing_assignment_targets_have_stable_parse_errors() {
    let cases = [("<?php\n$items[] ??= 'fallback';\n", 2, 10)];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported null coalescing assignment: only direct variable, direct array-offset, and direct object-property targets are implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_null_coalescing_expression_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$result = $value ?? 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior"
    );
}

#[test]
fn unsupported_expression_position_assignment_forms_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$items = [];\necho ($items['outer']['inner'] ??= 'value');\n",
            3,
            32,
            "unsupported null coalescing assignment: only direct variable, direct array-offset, and direct object-property targets are implemented",
        ),
        (
            "<?php\n$items = [];\necho ($items['outer']['inner'] = 'value');\n",
            3,
            32,
            "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented",
        ),
        (
            "<?php\n$items = [];\necho ($items[] ??= 'value');\n",
            3,
            16,
            "unsupported null coalescing assignment: only direct variable, direct array-offset, and direct object-property targets are implemented",
        ),
        (
            "<?php\n$items = [];\n$value = $items[] = 1;\n",
            3,
            10,
            "unsupported assignment expression: this chained assignment form is not implemented in the current subset",
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
fn emit_ir_rejects_assignment_expression_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$value = 1;\necho ($value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn unsupported_compound_assignments_have_stable_parse_errors() {
    let cases = [("<?php\n$items = [];\n$items[] += 2;\n", 3, 1)];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented; append offsets and nested targets are not implemented"
        );
    }
}

#[test]
fn compound_assignment_expressions_have_stable_parse_errors() {
    let error = parse_error(
        "<?php\n$items = ['outer' => ['inner' => 1]];\necho ($items['outer']['inner'] += 2);\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 33);
    assert_eq!(
        error.message,
        "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented; append offsets and nested targets are not implemented"
    );
}

#[test]
fn emit_ir_rejects_compound_assignment_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn unsupported_increment_decrement_operators_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$values = [[1]];\n++$values[0][0];\n",
            3,
            1,
            "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\n$values[0][0]--;\n",
            3,
            1,
            "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\necho ++$values[0][0];\n",
            3,
            6,
            "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\necho $values[0][0]--;\n",
            3,
            6,
            "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented",
        ),
        (
            "<?php\n$value = 1;\necho ++$value++;\n",
            3,
            6,
            "unsupported increment/decrement expression: chained increment/decrement expressions are not implemented",
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
fn unsupported_for_header_increment_decrement_targets_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$values = [[1]];\nfor (++$values[0][0]; false; ) {}\n",
            3,
            6,
        ),
        (
            "<?php\n$values = [[1]];\nfor ($values[0][0]--; false; ) {}\n",
            3,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_increment_decrement_expressions_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$value = 1;\necho $value++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn unsupported_foreach_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = [1];
FOREACH ($items as &$item) {
    echo $item;
}
"#,
            3,
            20,
            "unsupported foreach: by-reference iteration is not implemented; only by-value iteration is supported",
        ),
        (
            r#"<?php
$items = [[1]];
foreach ($items as [$item]) {
    echo $item;
}
"#,
            3,
            20,
            "unsupported foreach: destructuring loop targets are not implemented",
        ),
        (
            r#"<?php
$items = [[1]];
foreach ($items as $key => [$item]) {
    echo $item;
}
"#,
            3,
            28,
            "unsupported foreach: destructuring loop targets are not implemented",
        ),
        (
            r#"<?php
$items = [1];
echo foreach ($items as $item);
"#,
            3,
            6,
            "unsupported foreach: foreach is only supported as a statement in the current subset",
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
fn unsupported_for_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
for ($i = 0, $j = 0; $i < 3; $i = $i + 1) {
    echo $i;
}
"#,
            2,
            12,
            "unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot",
        ),
        (
            r#"<?php
for ($i = 0; $i < 3; $i = $i + 1, $j = $j + 1) {
    echo $i;
}
"#,
            2,
            33,
            "unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot",
        ),
        (
            r#"<?php
echo for ($i = 0; $i < 3; $i = $i + 1);
"#,
            2,
            6,
            "unsupported for: for loops are only supported as statements in the current subset",
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
fn do_while_expression_form_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
echo do {
    echo "tick";
} while (false);
"#,
            2,
            6,
        ),
        (
            r#"<?php
echo DO echo "tick"; WHILE (false);
"#,
            2,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported do-while: do-while loops are only supported as statements in the current subset"
        );
    }
}

#[test]
fn unsupported_switch_forms_are_rejected_with_stable_parse_error() {
    let cases = [(
        r#"<?php
echo switch ($value) {
    default:
        echo "fallback";
};
"#,
        2,
        6,
        "unsupported switch: switch is only supported as a statement in the current subset",
    )];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn malformed_alternate_switch_forms_have_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
$value = 2;
switch ($value):
    echo "body";
endswitch;
"#,
            4,
            5,
            "expected 'case', 'default', or 'endswitch' in alternate switch body",
        ),
        (
            r#"<?php
$value = 2;
switch ($value):
    case 1:
        echo "one";
"#,
            6,
            1,
            "expected 'endswitch' after alternate switch body",
        ),
        (
            r#"<?php
$value = 2;
switch ($value):
endswitch
"#,
            5,
            1,
            "expected ';' after endswitch",
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
fn unsupported_alternate_if_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$value = 2;
if ($value == 1) {
    echo "one";
} elseif ($value == 2):
    echo "two";
endif;
"#,
            5,
            23,
        ),
        (
            r#"<?php
if ($value) {
    echo "yes";
} ELSE:
    echo "no";
endif;
"#,
            4,
            7,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported if: alternate if/elseif/else colon/endif syntax is not implemented; use brace blocks or single-statement bodies"
        );
    }
}

#[test]
fn unsupported_break_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    break 2;
}
"#,
            3,
            5,
            "unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported",
        ),
        (
            r#"<?php
echo break;
"#,
            2,
            6,
            "unsupported break: break is only supported as a statement in the current subset",
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
fn unsupported_continue_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    continue 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported",
        ),
        (
            r#"<?php
while (true) {
    CONTINUE 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported",
        ),
        (
            r#"<?php
echo continue;
"#,
            2,
            6,
            "unsupported continue: continue is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
