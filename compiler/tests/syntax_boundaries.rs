use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_REFERENCE_ASSIGNMENT_REJECTION: &str = "LLVM reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_CLONE_REJECTION: &str = "LLVM clone lowering rejects clone expressions, including direct-variable clone assignments that mirror public and context-aware non-public property reference slots, until native object handles, property slot cloning, __clone dispatch, reference-slot metadata, references/copy-on-write, and exact native error behavior exist; phpc run handles current bounded clone behavior";
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
fn heredoc_and_nowdoc_strings_execute_current_unindented_label_subset() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$items = ["code" => 42];
$text = <<<TXT
hello {$name}
code: {$items['code']}
TXT;
$literal = <<<'TXT'
literal $name
TXT;
echo $text, $literal;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada\ncode: 42literal $name");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unsupported_heredoc_nowdoc_forms_have_stable_lex_errors() {
    let cases = [
        (
            "<?php\n$text = <<<TXT\nhello\n    TXT;\n",
            "unterminated heredoc/nowdoc string literal",
        ),
        (
            "<?php\n$text = <<<123\nhello\n123;\n",
            "unsupported heredoc/nowdoc string syntax: only unindented identifier labels are implemented; indentation stripping, label expressions, and malformed labels are not implemented",
        ),
    ];

    for (source, message) in cases {
        let error = lex_error(source);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 9);
        assert_eq!(error.message, message);
    }
}

#[test]
fn emit_ir_rejects_interpolated_heredoc_until_native_string_runtime_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\n$name = \"Ada\";\necho <<<TXT\nhello {$name}\nTXT;\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(error.message.contains("LLVM concatenation lowering"));
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
            "<?php\n['first' => $first] = [1];\n",
            2,
            2,
            "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\necho list($first);\n",
            2,
            6,
            "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\nlist($first[0]) = [1];\n",
            2,
            12,
            "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented",
        ),
        (
            "<?php\nlist(,) = [1, 2];\n",
            2,
            1,
            "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented",
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
    let error = php_compiler::emit_ir_source("<?php\n['first' => $first] = [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented"
    );
}

#[test]
fn unsupported_unset_forms_are_rejected_with_stable_parse_error() {
    let cases = [(
        r#"<?php
$items = [];
UNSET($items[]);
"#,
        3,
        13,
    )];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported unset: only direct variables like unset($name), direct array offset removal like unset($array[$key]), direct object properties like unset($object->property), and direct static property operands like unset(ClassName::$property) are implemented; dynamic property offsets, append, and broader nested unset forms are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_object_property_unset_lowering() {
    let error = php_compiler::emit_ir_source("<?php\nunset($box->name);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
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
fn emit_ir_rejects_magic_class_name_instantiation_after_parse() {
    let error = php_compiler::emit_ir_source("<?php\necho new self();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object/class lowering rejects"),
        "{}",
        error.message
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
fn unsupported_promoted_property_parameters_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass User {\n    public function __construct(public string $name) {}\n}\n",
            3,
            33,
        ),
        (
            "<?php\nclass User {\n    public function __construct(private $id) {}\n}\n",
            3,
            33,
        ),
        (
            "<?php\nclass User {\n    public function __construct(protected readonly string $name) {}\n}\n",
            3,
            33,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported promoted property parameter: constructor property promotion is not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_promoted_property_parameters_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass User {\n    public function __construct(public string $name) {}\n}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported promoted property parameter: constructor property promotion is not implemented"
    );
}

#[test]
fn unsupported_dnf_type_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nfunction accepts((Iterator&Countable)|ArrayAccess $value) {}\n",
            2,
            18,
        ),
        (
            "<?php\nfunction returns(): (Iterator&Countable)|ArrayAccess { return null; }\n",
            2,
            21,
        ),
        (
            "<?php\nclass Box {\n    public (Iterator&Countable)|ArrayAccess $value;\n}\n",
            3,
            12,
        ),
        (
            "<?php\nclass Box {\n    public static (Iterator&Countable)|ArrayAccess $value;\n}\n",
            3,
            19,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported DNF type declaration: parenthesized union/intersection type declarations are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_dnf_type_declarations_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nfunction accepts((Iterator&Countable)|ArrayAccess $value) {}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported DNF type declaration: parenthesized union/intersection type declarations are not implemented"
    );
}

#[test]
fn unsupported_readonly_class_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nreadonly class Value {\n    public $id;\n}\n",
            2,
            1,
            "unsupported readonly class declaration: readonly class metadata, typed-property enforcement, initialization and write rules, reflection, and native lowering are not implemented",
        ),
        (
            "<?php\nfinal readonly class Value {}\n",
            2,
            7,
            "unsupported readonly class declaration: readonly class metadata, typed-property enforcement, initialization and write rules, reflection, and native lowering are not implemented",
        ),
        (
            "<?php\nreadonly final class Value {}\n",
            2,
            1,
            "unsupported readonly class declaration: readonly class metadata, typed-property enforcement, initialization and write rules, reflection, and native lowering are not implemented",
        ),
        (
            "<?php\nreadonly readonly class Value {}\n",
            2,
            10,
            "duplicate readonly modifier in class declaration",
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
fn emit_ir_rejects_readonly_class_declarations_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\nfinal readonly class Value {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported readonly class declaration: readonly class metadata, typed-property enforcement, initialization and write rules, reflection, and native lowering are not implemented"
    );
}

#[test]
fn unsupported_readonly_property_declarations_have_stable_parse_errors() {
    let cases = [
        ("<?php\nclass Value {\n    public readonly $id;\n}\n", 3, 12),
        (
            "<?php\nclass Value {\n    public readonly string $id;\n}\n",
            3,
            12,
        ),
        (
            "<?php\nclass Value {\n    private static readonly int $id;\n}\n",
            3,
            20,
        ),
        (
            "<?php\nclass Value {\n    readonly public string $id;\n}\n",
            3,
            5,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported readonly property declaration: readonly property metadata, initialization rules, write-once enforcement, reflection, and native lowering are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_readonly_property_declarations_at_parse_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    public readonly string $id;\n}\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported readonly property declaration: readonly property metadata, initialization rules, write-once enforcement, reflection, and native lowering are not implemented"
    );
}

#[test]
fn unsupported_readonly_non_property_class_members_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Value {\n    readonly function id() {}\n}\n",
            3,
            5,
        ),
        (
            "<?php\nclass Value {\n    public readonly function id() {}\n}\n",
            3,
            12,
        ),
        (
            "<?php\nclass Value {\n    readonly const ID = 1;\n}\n",
            3,
            5,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported readonly class member modifier: readonly methods and readonly class constants are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_readonly_non_property_class_members_at_parse_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    readonly const ID = 1;\n}\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported readonly class member modifier: readonly methods and readonly class constants are not implemented"
    );
}

#[test]
fn abstract_and_final_methods_remain_supported_member_metadata() {
    let execution = run_source(
        "<?php\nabstract class Base {\n    abstract protected function compute();\n}\nfinal class Leaf {\n    public final function compute() {}\n}\necho 'ok';\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn abstract_final_method_combinations_keep_stable_parse_error() {
    let error = parse_error("<?php\nclass Base {\n    abstract final function compute();\n}\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 14);
    assert_eq!(
        error.message,
        "unsupported class member modifier combination: abstract final methods are not implemented"
    );
}

#[test]
fn unsupported_abstract_final_property_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Value {\n    abstract $id;\n}\n",
            3,
            5,
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    final $id;\n}\n",
            3,
            5,
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    public final $id;\n}\n",
            3,
            12,
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    abstract final $id;\n}\n",
            3,
            5,
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
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
fn unsupported_abstract_final_class_constant_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Value {\n    abstract const ID = 1;\n}\n",
            3,
            5,
            "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    final const ID = 1;\n}\n",
            3,
            5,
            "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    public final const ID = 1;\n}\n",
            3,
            12,
            "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    abstract final const ID = 1;\n}\n",
            3,
            5,
            "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented",
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
fn unsupported_typed_property_declarations_keep_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Value {\n    public string $id;\n}\n",
            3,
            12,
            "unsupported property type declaration: typed property storage and enforcement are not implemented",
        ),
        (
            "<?php\nclass Value {\n    public static int $id;\n}\n",
            3,
            19,
            "unsupported static property type declaration: typed static property metadata, uninitialized state, and write enforcement are not implemented",
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
fn unsupported_asymmetric_property_visibility_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Value {\n    public private(set) string $id;\n}\n",
            3,
            12,
        ),
        (
            "<?php\nclass Value {\n    protected private(SET) string $id;\n}\n",
            3,
            15,
        ),
        (
            "<?php\nclass Value {\n    public static protected(set) string $id;\n}\n",
            3,
            19,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported asymmetric property visibility: PHP 8 set-visibility modifiers such as private(set) and protected(set) require property visibility metadata, typed-property storage and enforcement, reflection behavior, and native lowering"
        );
    }
}

#[test]
fn emit_ir_rejects_asymmetric_property_visibility_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Value {\n    public private(set) string $id;\n}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported asymmetric property visibility: PHP 8 set-visibility modifiers such as private(set) and protected(set) require property visibility metadata, typed-property storage and enforcement, reflection behavior, and native lowering"
    );
}

#[test]
fn emit_ir_rejects_abstract_final_non_method_members_at_parse_boundary() {
    let property_error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    abstract $id;\n}\n").unwrap_err();

    assert_eq!(property_error.phase, Phase::Parse);
    assert_eq!(
        property_error.message,
        "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented"
    );

    let const_error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    final const ID = 1;\n}\n")
            .unwrap_err();

    assert_eq!(const_error.phase, Phase::Parse);
    assert_eq!(
        const_error.message,
        "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented"
    );
}

#[test]
fn malformed_clone_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$copy = clone;\n",
            2,
            14,
            "expected expression, found ;",
        ),
        (
            "<?php\necho clone;\n",
            2,
            11,
            "expected expression, found ;",
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
fn emit_ir_rejects_clone_expression_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$copy = clone $object;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CLONE_REJECTION);
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
fn unsupported_nullsafe_object_operator_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$name = $user?->name;\n",
            2,
            14,
            "unsupported nullsafe object operator: ?-> property and method access is not implemented",
        ),
        (
            "<?php\necho $user?->name;\n",
            2,
            11,
            "unsupported nullsafe object operator: ?-> property and method access is not implemented",
        ),
        (
            "<?php\n$value = $user?->profile();\n",
            2,
            15,
            "unsupported nullsafe object operator: ?-> property and method access is not implemented",
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
fn emit_ir_rejects_nullsafe_object_operator_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$name = $user?->name;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported nullsafe object operator: ?-> property and method access is not implemented"
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
            "<?php\n$items = [];\necho ($items[]['inner'] = 'value');\n",
            3,
            25,
            "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented",
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
fn emit_ir_rejects_reference_assignment_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$value = 1;\n$alias =& $value;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_REFERENCE_ASSIGNMENT_REJECTION);
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
            "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented; append offsets and nested variable targets are not implemented"
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
        "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented; append offsets and nested variable targets are not implemented"
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
            "unsupported increment/decrement target: only direct static variables, direct array/object offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer and float values; append offsets and nested variable targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\n$values[0][0]--;\n",
            3,
            1,
            "unsupported increment/decrement target: only direct static variables, direct array/object offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer and float values; append offsets and nested variable targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\necho ++$values[0][0];\n",
            3,
            6,
            "unsupported increment/decrement target: only direct static variables, direct array/object offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer and float values; append offsets and nested variable targets are not implemented",
        ),
        (
            "<?php\n$values = [[1]];\necho $values[0][0]--;\n",
            3,
            6,
            "unsupported increment/decrement target: only direct static variables, direct array/object offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer and float values; append offsets and nested variable targets are not implemented",
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
            "unsupported increment/decrement target: only direct static variables, direct array/object offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer and float values; append offsets and nested variable targets are not implemented"
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
foreach ($items as &$key => $item) {
    echo $item;
}
"#,
            3,
            1,
            "unsupported foreach: key variables cannot be by-reference in the current subset",
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
    let cases = [(
        r#"<?php
echo for ($i = 0; $i < 3; $i = $i + 1);
"#,
        2,
        6,
        "unsupported for: for loops are only supported as statements in the current subset",
    )];

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
    break $depth;
}
"#,
            3,
            5,
            "unsupported break: only positive integer loop-depth literals are implemented in the current subset",
        ),
        (
            r#"<?php
while (true) {
    break 0;
}
"#,
            3,
            11,
            "unsupported break: loop-depth must be a positive integer literal in the current subset",
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
    continue $depth;
}
"#,
            3,
            5,
            "unsupported continue: only positive integer loop-depth literals are implemented in the current subset",
        ),
        (
            r#"<?php
while (true) {
    continue 0;
}
"#,
            3,
            14,
            "unsupported continue: loop-depth must be a positive integer literal in the current subset",
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
