use php_compiler::ast::{Expr, NewClassName, Stmt};
use php_compiler::codegen::emit_native_executable_c_source;
use php_compiler::error::Phase;
use php_compiler::{parse, run_source};

const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment outside lowerable direct variables, null coalescing assignment, increment/decrement, non-direct assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_INSTANCEOF_REJECTION: &str = "LLVM instanceof lowering rejects class/interface relationship checks until native class metadata tables, object handles, inheritance/interface registries, class-name resolution, autoload interaction, references/copy-on-write, and exact native instanceof diagnostics exist; phpc run handles current bounded instanceof behavior";
const LLVM_CLONE_REJECTION: &str = "LLVM clone lowering rejects clone expressions, including direct-variable clone assignments that mirror public and context-aware non-public property reference slots, until native object handles, property slot cloning, __clone dispatch, reference-slot metadata, references/copy-on-write, and exact native error behavior exist; phpc run handles current bounded clone behavior";
const LLVM_INTERFACE_REJECTION: &str = "LLVM interface lowering rejects interface declarations until native class/interface tables, implementation checks, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current interface metadata behavior";
const LLVM_TRAIT_REJECTION: &str = "LLVM trait lowering rejects trait declarations until native trait tables, class trait-use composition, conflict resolution, aliasing, relationship metadata, autoload interaction, and exact native error behavior exist; phpc run handles current trait metadata behavior";
const LLVM_ENUM_REJECTION: &str = "LLVM enum lowering rejects enum declarations until native class/enum tables, enum case objects, backed enum values, interface implementation, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current enum metadata behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_GLOBAL_CONSTANT_REJECTION: &str = "LLVM global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const LLVM_ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const LLVM_MATCH_REJECTION: &str = "LLVM match expression lowering rejects match expressions until native strict arm comparison, default/exhaustiveness handling, value evaluation order, references/copy-on-write, and exact native error behavior exist; phpc run handles current match expression behavior";
const LLVM_NONLOCAL_UNSET_REJECTION: &str = "LLVM native array non-local unset lowering rejects object, dynamic-object, non-direct object, and static property unsets until non-local owner cells, magic __unset dispatch, typed/static property state, references/copy-on-write, and exact diagnostics share one unset owner contract; local variables and native array offset unsets use their shared native lvalue unset contracts";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
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
fn short_echo_tags_execute_as_echo_statements() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
?>
<?= $name ?>|<?= "B", "C" ?>"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|BC");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_lowers_short_echo_tags_through_existing_echo_path() {
    let ir = php_compiler::emit_ir_source("<?= \"native\" ?>").unwrap();

    assert!(ir.contains("c\"native\\00\""), "{ir}");
    assert!(!ir.contains("short echo"), "{ir}");
}

#[test]
fn php_closing_tag_terminates_semicolonless_final_statements() {
    let execution = run_source(
        r#"<?php
$value = "assigned"?>|<?php
print "printed";?>|<?php
echo $value?>"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "|printed|assigned");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_attribute_syntax_accepts_current_runtime_metadata_noop_boundary() {
    let cases = [
        "<?php\n#[Route('/wp-json/demo')]\nfunction handler() {}\n",
        "<?php\nclass Box {\n    #[Inject]\n    public $service;\n}\n",
        "<?php\nclass Box {\n    #[Inject('service')]\n    public $service;\n}\n",
    ];

    for source in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }
}

#[test]
fn emit_ir_rejects_php_attributes_at_lex_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n#[Hook('init')]\nfunction boot() {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn unsupported_backtick_execution_operator_has_stable_lex_errors() {
    let cases = [
        ("<?php\n$output = `whoami`;\n", 2, 11),
        ("<?php\necho `printf {$name}`;\n", 2, 6),
    ];

    for (source, line, column) in cases {
        let error = lex_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported backtick execution operator: shell command execution, interpolation, process I/O, error handling, platform behavior, references/copy-on-write, and native lowering are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_backtick_execution_operator_at_lex_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$output = `whoami`;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Lex);
    assert_eq!(
        error.message,
        "unsupported backtick execution operator: shell command execution, interpolation, process I/O, error handling, platform behavior, references/copy-on-write, and native lowering are not implemented"
    );
}

#[test]
fn emit_ir_rejects_interpolated_heredoc_until_native_string_runtime_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\n$name = \"Ada\";\necho <<<TXT\nhello {$name}\nTXT;\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(error.message.contains("LLVM interpolated-string lowering"));
}

#[test]
fn unsupported_array_item_forms_are_rejected_with_stable_parse_error() {
    let cases = [(
        r#"<?php
$key = "name";
$items = [&$key => "value"];
"#,
        3,
        12,
        "unsupported array reference key: reference keys are not implemented",
    )];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_array_destructuring_assignments_have_stable_parse_errors() {
    let unsupported = "unsupported array destructuring: statement-form list(...) = expr and [...] = expr support variable, skipped, nested, and literal int/string keyed targets; dynamic-key and complex writable targets outside direct variables and nested lists are not implemented";
    let cases = [
        ("<?php\n[$key => $value] = $items;\n", 2, 2, unsupported),
        ("<?php\nlist($first[0]) = [1];\n", 2, 12, unsupported),
        (
            "<?php\necho list($first);\n",
            2,
            6,
            "syntax error, unexpected token \")\", expecting \"=\"",
        ),
        (
            "<?php\nlist(,) = [1, 2];\n",
            2,
            1,
            "php fatal: Cannot use empty list",
        ),
        (
            "<?php\nlist(&$first) = [1];\n",
            2,
            6,
            "php fatal: Assignments can only happen to writable values",
        ),
        (
            "<?php\n[$first, 'second' => $second] = [1, 2];\n",
            2,
            1,
            "php fatal: Cannot mix keyed and unkeyed array entries in assignments",
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
    let error = php_compiler::emit_ir_source("<?php\nlist(&$first) = [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "php fatal: Assignments can only happen to writable values"
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
            "unsupported unset: supported operands are direct variables, direct/nested array offsets, direct or bounded non-direct object properties and object-property array offsets, and direct static properties; append unset, object operators after array offsets, and broader dynamic expression roots are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_object_property_unset_lowering() {
    let error = php_compiler::emit_ir_source("<?php\nunset($box->name);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_NONLOCAL_UNSET_REJECTION);
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
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
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
fn emit_ir_rejects_yield_from_syntax_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\nfunction gen() { yield from [1, 2]; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn match_expression_executes_current_strict_runtime_subset() {
    let execution = run_source(
        r#"<?php
$status = 200;
$probe = 0;
echo match ($status) {
    '200' => $probe = 99,
    201, 200 => 'ok',
    default => 'other',
}, "\n";
echo MATCH ("x") {
    "y" => "bad",
    default => "default",
}, "\n";
echo $probe, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok\ndefault\n0\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_match_expression_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\n$value = match (200) {\n    default => 'other',\n};\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MATCH_REJECTION);
}

#[test]
fn non_bcmath_exponentiation_has_stable_runtime_boundaries() {
    let cases = [
        (
            "<?php\necho 2 ** 3;\n",
            2,
            6,
            "unsupported call operator **: non-BcMath\\Number exponentiation is not implemented in the current subset",
        ),
        (
            "<?php\n$value = 2;\n$value **= 3;\n",
            3,
            1,
            "unsupported call operator **=: non-BcMath\\Number exponentiation assignment is not implemented in the current subset",
        ),
        (
            "<?php\n$value = 2;\necho ($value **= 3);\n",
            3,
            7,
            "unsupported call operator **=: non-BcMath\\Number exponentiation assignment is not implemented in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = runtime_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn emit_ir_rejects_non_bcmath_exponentiation_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\necho 2 ** 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARITHMETIC_REJECTION);
}

#[test]
fn first_class_callable_syntax_executes_current_runtime_subset() {
    let cases = [
        ("<?php\n$callback = strlen(...);\n", ""),
        (
            "<?php\n$callback = 'strlen';\necho $callback(...);\n",
            "strlen",
        ),
    ];

    for (source, stdout) in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, stdout);
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }
}

#[test]
fn emit_ir_rejects_first_class_callable_syntax_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$callback = strlen(...);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn spread_arguments_parse_as_source_ordered_call_argument_nodes() {
    let program = parse(
        "<?php\nfunction handler($first, $tail, $named) {}\n$args = ['tail'];\nhandler('first', ...$args, named: 'named');\n",
    )
    .unwrap();

    let Stmt::Expr {
        expr: Expr::Call { args, .. },
        ..
    } = &program.statements[2]
    else {
        panic!(
            "expected direct call statement, got {:#?}",
            program.statements[2]
        );
    };

    assert_eq!(args.len(), 3);
    assert!(matches!(args[0], Expr::String(ref value, _) if value == "first"));
    match &args[1] {
        Expr::SpreadArgument { expr, .. } => {
            assert!(matches!(expr.as_ref(), Expr::Variable(name, _) if name == "args"));
        }
        other => panic!("expected second source argument to be spread, got {other:#?}"),
    }
    match &args[2] {
        Expr::NamedArgument { name, expr, .. } => {
            assert_eq!(name, "named");
            assert!(matches!(expr.as_ref(), Expr::String(value, _) if value == "named"));
        }
        other => panic!("expected third source argument to be named, got {other:#?}"),
    }
}

#[test]
fn native_codegen_lowers_argument_unpacking_through_shared_finalization_bridge() {
    let cases = [
        (
            "<?php\nclass Handler { public function run($first, $value) {} }\n$handler = new Handler();\n$args = ['init'];\n$handler->run('first', ...$args);\n",
            1,
        ),
        (
            "<?php\nclass Handler { public static function run($first, $value) {} }\n$args = ['init'];\nHandler::run('first', ...$args);\n",
            1,
        ),
    ];

    for (source, source_index) in cases {
        let program = parse(source).unwrap();
        let generated = emit_native_executable_c_source(&program).unwrap();

        assert!(
            generated.contains("phpc_native_materialized_call_arguments_new")
                && generated.contains("phpc_native_materialized_call_arguments_unpack_array_value_and_free")
                && generated.contains("phpc_native_materialized_call_arguments_finalize_with_diagnostic"),
            "spread source slot {source_index} should lower through materialized call arguments:\n{generated}"
        );
        assert!(
            !generated.contains("spread call argument at source slot"),
            "spread source slot {source_index} should not hit the old finalization blocker:\n{generated}"
        );
    }
}

#[test]
fn native_codegen_lowers_descriptor_closure_spread_through_unpacked_handle_bridge() {
    let program = parse(
        "<?php\n$closure = function ($first, $value) { return $first . $value; };\n$args = ['tail'];\necho $closure('head', ...$args);\n",
    )
    .unwrap();

    let generated = emit_native_executable_c_source(&program).unwrap();

    assert!(
        generated.contains("phpc_native_materialized_call_arguments_new")
            && generated
                .contains("phpc_native_materialized_call_arguments_unpack_array_value_and_free")
            && generated
                .contains("phpc_native_materialized_call_arguments_finalize_with_diagnostic")
            && generated
                .contains("phpc_native_closure_invoke_value_with_diagnostic_and_free_arguments"),
        "descriptor closure spread should lower through materialized call arguments:\n{generated}"
    );
    assert!(!generated.contains("spread operands need a materialized-entry producer"));
}

#[test]
fn unsupported_call_time_reference_arguments_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nfunction handler($value) {}\n$value = 1;\nhandler(&$value);\n",
            4,
            9,
        ),
        (
            "<?php\nclass Hooks { public function add($hook) {} }\n$hooks = new Hooks();\n$hook = 'init';\n$hooks->add(&$hook);\n",
            5,
            13,
        ),
        (
            "<?php\nclass Hooks { public static function add($hook) {} }\n$hook = 'init';\nHooks::add(&$hook);\n",
            4,
            12,
        ),
        (
            "<?php\nclass Hook { public function __construct($hook) {} }\n$hook = 'init';\n$instance = new Hook(&$hook);\n",
            4,
            22,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported call-time by-reference argument: passing & at a call site requires legacy syntax handling, by-reference parameter metadata, alias setup, default handling, variadic/unpacking interaction, references/copy-on-write, and native lowering"
        );
    }
}

#[test]
fn emit_ir_rejects_call_time_reference_arguments_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nfunction handler($value) {}\n$value = 1;\nhandler(&$value);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported call-time by-reference argument: passing & at a call site requires legacy syntax handling, by-reference parameter metadata, alias setup, default handling, variadic/unpacking interaction, references/copy-on-write, and native lowering"
    );
}

#[test]
fn static_arrow_functions_parse_and_can_be_stored() {
    php_compiler::parse("<?php\n$handler = static fn ($value) => $value;\n").unwrap();
    php_compiler::parse("<?php\necho static fn () => 1;\n").unwrap();

    let execution = run_source(
        "<?php\n$handler = static fn ($value) => $value;\necho $handler ? \"stored\" : \"missing\";\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "stored");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn named_arguments_parse_as_source_ordered_call_argument_nodes() {
    let program =
        parse("<?php\nfunction greet($name, $punct = '!') {}\ngreet(punct: '?', name: 'Ada');\n")
            .unwrap();

    let Stmt::Expr {
        expr: Expr::Call { args, .. },
        ..
    } = &program.statements[1]
    else {
        panic!(
            "expected direct call statement, got {:#?}",
            program.statements[1]
        );
    };

    assert_eq!(args.len(), 2);
    match &args[0] {
        Expr::NamedArgument { name, expr, .. } => {
            assert_eq!(name, "punct");
            assert!(matches!(expr.as_ref(), Expr::String(value, _) if value == "?"));
        }
        other => panic!("expected first source argument to be named, got {other:#?}"),
    }
    match &args[1] {
        Expr::NamedArgument { name, expr, .. } => {
            assert_eq!(name, "name");
            assert!(matches!(expr.as_ref(), Expr::String(value, _) if value == "Ada"));
        }
        other => panic!("expected second source argument to be named, got {other:#?}"),
    }
}

#[test]
fn emit_ir_rejects_named_builtin_arguments_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\necho strlen(string: 'abc');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(error
        .message
        .contains("named argument lowering is only implemented"));
}

#[test]
fn emit_ir_rejects_magic_class_name_instantiation_after_parse() {
    let error = php_compiler::emit_ir_source("<?php\necho new self();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error
            .message
            .contains("object-instantiation lowering rejects"),
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
fn parenthesized_dynamic_new_class_expressions_reach_runtime_boundaries() {
    let cases = [
        (
            "<?php\n$class = \"Box\";\n$box = new ($class)();\n",
            "Class \"Box\" not found",
        ),
        (
            "<?php\necho new (factory())();\n",
            "Call to undefined function factory()",
        ),
    ];

    for (source, message) in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(
            execution.exit_code, 255,
            "parenthesized dynamic new should reach a runtime fatal boundary"
        );
        assert!(execution.stdout.contains(message), "{}", execution.stdout);
        assert_eq!(execution.stderr, "");
    }
}

#[test]
fn emit_ir_rejects_parenthesized_dynamic_new_class_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$class = \"Box\";\n$box = new ($class)();\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn promoted_property_parameters_are_limited_to_constructor_contexts() {
    let cases = [
        ("<?php\nfunction make(public string $name) {}\n", 2, 15),
        (
            "<?php\nclass User {\n    public function set(public string $name) {}\n}\n",
            3,
            25,
        ),
        ("<?php\n$fn = function (public string $name) {};\n", 2, 17),
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
fn readonly_promoted_property_parameters_parse_for_startup_diagnostics() {
    let execution = run_source(
        "<?php\nclass User {\n    public function __construct(protected readonly string $name) {}\n}\n",
    )
    .unwrap();
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_promoted_property_parameters_at_parse_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\nfunction make(public string $name) {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported promoted property parameter: constructor property promotion is not implemented"
    );
}

#[test]
fn dnf_type_declarations_parse_and_run_in_current_metadata_subset() {
    let cases = [
        "<?php\nfunction accepts((Iterator&Countable)|ArrayAccess $value) {}\n",
        "<?php\nfunction returns(): (Iterator&Countable)|ArrayAccess { return null; }\n",
    ];

    for source in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }
}

#[test]
fn emit_ir_rejects_dnf_type_declarations_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nfunction accepts((Iterator&Countable)|ArrayAccess $value) {}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn unsupported_grouped_use_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nuse App\\{Controller, Service};\n",
            2,
            9,
            "unsupported grouped use declaration: grouped class, function, and const imports are not implemented",
        ),
        (
            "<?php\nuse {App\\Controller};\n",
            2,
            5,
            "unsupported grouped use declaration: grouped class, function, and const imports are not implemented",
        ),
        (
            "<?php\nuse function App\\{make, build};\n",
            2,
            18,
            "unsupported grouped use declaration: grouped class, function, and const imports are not implemented",
        ),
        (
            "<?php\nuse const App\\{VALUE, OTHER};\n",
            2,
            15,
            "unsupported grouped use declaration: grouped class, function, and const imports are not implemented",
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
fn emit_ir_rejects_grouped_use_declarations_at_parse_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\nuse App\\{Controller, Service};\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported grouped use declaration: grouped class, function, and const imports are not implemented"
    );
}

#[test]
fn fully_qualified_function_calls_parse_as_exact_global_calls() {
    let program = parse("<?php\n$result = \\strlen('abc');\n$other = \\App\\make();\n").unwrap();

    assert_eq!(program.statements.len(), 2);
    match &program.statements[0] {
        Stmt::Assign {
            expr: Expr::Call { name, args, span },
            ..
        } => {
            assert_eq!(name, "\\strlen");
            assert_eq!(args.len(), 1);
            assert_eq!((span.line, span.column), (2, 11));
        }
        other => panic!("expected fully-qualified strlen call assignment, got {other:?}"),
    }
    match &program.statements[1] {
        Stmt::Assign {
            expr: Expr::Call { name, args, span },
            ..
        } => {
            assert_eq!(name, "\\App\\make");
            assert!(args.is_empty());
            assert_eq!((span.line, span.column), (3, 10));
        }
        other => panic!("expected fully-qualified namespaced call assignment, got {other:?}"),
    }
}

#[test]
fn qualified_function_calls_parse_to_resolved_direct_call_names() {
    let program = parse(
        r#"<?php
namespace Root;
$a = App\helper();
$b = Sub\helper();
namespace\helper();
$d = \App\helper();
$e = \strlen();
"#,
    )
    .unwrap();

    let names: Vec<&str> = program
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Assign {
                expr: Expr::Call { name, .. },
                ..
            }
            | Stmt::Expr {
                expr: Expr::Call { name, .. },
                ..
            } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(
        names,
        [
            "Root\\App\\helper",
            "Root\\Sub\\helper",
            "Root\\helper",
            "\\App\\helper",
            "\\strlen"
        ]
    );
}

#[test]
fn namespace_qualified_constant_reads_parse_to_exact_global_constant_names() {
    let program = parse(
        r#"<?php
namespace Root;
$a = App\VERSION;
$b = namespace\VERSION;
$c = \PHP_VERSION;
"#,
    )
    .unwrap();

    let names: Vec<&str> = program
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Assign {
                expr: Expr::GlobalConstant { name, .. },
                ..
            } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(
        names,
        ["\\Root\\App\\VERSION", "\\Root\\VERSION", "\\PHP_VERSION"]
    );

    let execution = run_source("<?php\necho \\PHP_VERSION;\n").unwrap();
    assert_eq!(execution.stdout, "8.3.0");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_fully_qualified_constant_reads_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\necho \\PHP_VERSION;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_GLOBAL_CONSTANT_REJECTION);
}

#[test]
fn readonly_class_declarations_parse_for_startup_diagnostics() {
    for source in [
        "<?php\nfinal readonly class Value {}\n",
        "<?php\nreadonly final class Value {}\n",
    ] {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }

    let fatal = run_source("<?php\nreadonly class Value {\n    public $id;\n}\n").unwrap();
    assert_eq!(fatal.stdout, "");
    assert_eq!(fatal.exit_code, 255);
    assert_eq!(
        fatal.stderr,
        "Fatal error: Readonly property Value::$id must have type in Command line code on line 3"
    );

    let duplicate = parse_error("<?php\nreadonly readonly class Value {}\n");
    assert_eq!(duplicate.line, 2);
    assert_eq!(duplicate.column, 10);
    assert_eq!(
        duplicate.message,
        "duplicate readonly modifier in class declaration"
    );
}

#[test]
fn emit_ir_rejects_readonly_class_declarations_until_native_metadata_exists() {
    let error = php_compiler::emit_ir_source("<?php\nfinal readonly class Value {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn readonly_property_declarations_parse_for_startup_diagnostics() {
    let valid = run_source("<?php\nclass Value {\n    public readonly string $id;\n}\n").unwrap();
    assert_eq!(valid.stdout, "");
    assert_eq!(valid.stderr, "");
    assert_eq!(valid.exit_code, 0);

    let cases = [
        (
            "<?php\nclass Value {\n    public readonly $id;\n}\n",
            "Fatal error: Readonly property Value::$id must have type in Command line code on line 3",
        ),
        (
            "<?php\nclass Value {\n    private static readonly int $id;\n}\n",
            "Fatal error: Static property Value::$id cannot be readonly in Command line code on line 3",
        ),
        (
            "<?php\nclass Value {\n    readonly public string $id = 'x';\n}\n",
            "Fatal error: Readonly property Value::$id cannot have default value in Command line code on line 3",
        ),
    ];

    for (source, message) in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.exit_code, 255);
        assert_eq!(execution.stderr, message);
    }
}

#[test]
fn emit_ir_rejects_readonly_property_declarations_until_native_metadata_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    public readonly string $id;\n}\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn readonly_non_property_class_members_parse_for_startup_diagnostics() {
    let cases = [
        (
            "<?php\nclass Value {\n    readonly function id() {}\n}\n",
            "Fatal error: Cannot use the readonly modifier on a method in Command line code on line 3",
        ),
        (
            "<?php\nclass Value {\n    public readonly function id() {}\n}\n",
            "Fatal error: Cannot use the readonly modifier on a method in Command line code on line 3",
        ),
        (
            "<?php\nclass Value {\n    readonly const ID = 1;\n}\n",
            "Fatal error: Cannot use the readonly modifier on a class constant in Command line code on line 3",
        ),
    ];

    for (source, message) in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.exit_code, 255);
        assert_eq!(execution.stderr, message);
    }
}

#[test]
fn emit_ir_rejects_readonly_non_property_class_members_until_native_metadata_exists() {
    let error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    readonly const ID = 1;\n}\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
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
fn duplicate_and_conflicting_class_modifiers_report_php_startup_fatals() {
    let cases = [
        (
            "<?php\nclass Base {\n    public public function compute() {}\n}\n",
            "Fatal error: Multiple access type modifiers are not allowed in Command line code on line 3",
        ),
        (
            "<?php\nclass Base {\n    static static function compute() {}\n}\n",
            "Fatal error: Multiple static modifiers are not allowed in Command line code on line 3",
        ),
        (
            "<?php\nclass Base {\n    abstract abstract function compute() {}\n}\n",
            "Fatal error: Multiple abstract modifiers are not allowed in Command line code on line 3",
        ),
        (
            "<?php\nclass Base {\n    final final function compute() {}\n}\n",
            "Fatal error: Multiple final modifiers are not allowed in Command line code on line 3",
        ),
        (
            "<?php\nfinal final class Base {}\n",
            "Fatal error: Multiple final modifiers are not allowed in Command line code on line 2",
        ),
        (
            "<?php\nclass Base {\n    abstract final function compute();\n}\n",
            "Fatal error: Cannot use the final modifier on an abstract method in Command line code on line 3",
        ),
        (
            "<?php\nfinal abstract class Base {}\n",
            "Fatal error: Cannot use the final modifier on an abstract class in Command line code on line 2",
        ),
    ];

    for (source, message) in cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "");
        assert_eq!(execution.stderr, message);
        assert_eq!(execution.exit_code, 255);
    }
}

#[test]
fn abstract_final_property_declarations_keep_bounded_diagnostics() {
    let cases = [
        (
            "<?php\nclass Value {\n    abstract $id;\n}\n",
            "php fatal: Only hooked properties may be declared abstract",
        ),
        (
            "<?php\nclass Value {\n    final $id;\n}\n",
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    public final $id;\n}\n",
            "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented",
        ),
        (
            "<?php\nclass Value {\n    abstract final $id;\n}\n",
            "php fatal: Only hooked properties may be declared abstract",
        ),
    ];

    for (source, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, 3);
        assert_eq!(error.message, message);
    }
}

#[test]
fn final_class_constant_declarations_parse_for_startup_validation() {
    parse(
        "<?php\nclass Value {\n    final const ID = 1;\n    public final const NAME = 'value';\n}\ninterface Contract {\n    final public const TOKEN = 'contract';\n}\n",
    )
    .unwrap();
}

#[test]
fn parenthesized_dnf_typed_property_declarations_parse_for_metadata() {
    let cases = [
        "<?php\nclass Value {\n    public (Countable&Iterator)|ArrayAccess $id;\n}\n",
        "<?php\nclass Value {\n    public static (Countable&Iterator)|ArrayAccess $id;\n}\n",
    ];

    for source in cases {
        parse(source).unwrap();
    }
}

#[test]
fn unsupported_property_hook_declarations_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\nclass Post {\n    public string $title { get => $this->title; }\n}\n",
            3,
            26,
        ),
        (
            "<?php\nclass Post {\n    public $title { get => 'draft'; }\n}\n",
            3,
            19,
        ),
        (
            "<?php\nclass Post {\n    protected string $slug { set { $this->slug = $value; } }\n}\n",
            3,
            28,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported property hook declaration: PHP property get/set hooks require hook metadata, backing/virtual property behavior, typed-property storage and enforcement, references, reflection, and native lowering"
        );
    }
}

#[test]
fn interface_property_hook_declarations_parse_as_interface_metadata() {
    let execution = run_source(
        "<?php\ninterface Contract {\n    public mixed $value { get; }\n}\necho \"Done\";\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "Done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_property_hook_declarations_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Post {\n    public string $title { get => $this->title; }\n}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported property hook declaration: PHP property get/set hooks require hook metadata, backing/virtual property behavior, typed-property storage and enforcement, references, reflection, and native lowering"
    );
}

#[test]
fn asymmetric_property_visibility_declaration_diagnostics_are_php_shaped() {
    let cases = [
        (
            "<?php\nclass Value {\n    public private(set) $id;\n}\n",
            3,
            "php fatal: Property with asymmetric visibility Value::$id must have type",
        ),
        (
            "<?php\nclass Value {\n    private protected(set) string $id;\n}\n",
            3,
            "php fatal: Visibility of property Value::$id must not be weaker than set visibility",
        ),
        (
            "<?php\nclass Value {\n    private(set) protected(set) string $id;\n}\n",
            3,
            "php fatal: Multiple access type modifiers are not allowed",
        ),
        (
            "<?php\nclass Value {\n    public private(set) int $id { get => 42; }\n}\n",
            3,
            "php fatal: get-only virtual property Value::$id must not specify asymmetric visibility",
        ),
        (
            "<?php\nclass Value {\n    public private(set) int $id { set {} }\n}\n",
            3,
            "php fatal: set-only virtual property Value::$id must not specify asymmetric visibility",
        ),
    ];

    for (source, line, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.message, message);
    }
}

#[test]
fn asymmetric_property_visibility_valid_declarations_parse_for_runtime_metadata() {
    let execution = run_source(
        "<?php\nclass Value {\n    public private(set) string $id;\n    private private(set) string $secret;\n    protected protected(set) string $token;\n}\necho \"Done\";\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "Done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_asymmetric_property_visibility_after_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Value {\n    public private(set) string $id;\n}\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_abstract_non_method_members_at_parse_boundary() {
    let property_error =
        php_compiler::emit_ir_source("<?php\nclass Value {\n    abstract $id;\n}\n").unwrap_err();

    assert_eq!(property_error.phase, Phase::Parse);
    assert_eq!(
        property_error.message,
        "php fatal: Only hooked properties may be declared abstract"
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
fn dynamic_instanceof_targets_parse_as_class_name_operands() {
    let program = parse(
        "<?php\n$is = $object instanceof $class;\n$expr = $object INSTANCEOF (target_name());\n",
    )
    .unwrap();

    let Stmt::Assign { expr: first, .. } = &program.statements[0] else {
        panic!("first statement should be an assignment: {program:?}");
    };
    let Expr::InstanceOf {
        class_name: NewClassName::DynamicVariable(variable),
        ..
    } = first
    else {
        panic!("first instanceof target should be a dynamic variable: {first:?}");
    };
    assert_eq!(variable, "class");

    let Stmt::Assign { expr: second, .. } = &program.statements[1] else {
        panic!("second statement should be an assignment: {program:?}");
    };
    let Expr::InstanceOf {
        class_name: NewClassName::DynamicExpression(target),
        ..
    } = second
    else {
        panic!("second instanceof target should be a dynamic expression: {second:?}");
    };
    assert!(matches!(target.as_ref(), Expr::Call { name, .. } if name == "target_name"));
}

#[test]
fn emit_ir_rejects_instanceof_expression_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$value = 7;\n$is = $value instanceof self;\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INSTANCEOF_REJECTION);
}

#[test]
fn unsupported_interface_declaration_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\ninterface Renderable {\n    public const string NAME = \"view\";\n}\n",
            3,
            18,
            "unsupported interface constant declaration: typed interface constants are not implemented",
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
            "<?php\ntrait Reusable {\n    final public function render() {}\n}\n",
            3,
            18,
            "unsupported trait method declaration: final trait methods, __TRAIT__ context, references/copy-on-write, and native lowering remain unsupported",
        ),
        (
            "<?php\ntrait Reusable {\n    final protected static function render() {}\n}\n",
            3,
            28,
            "unsupported trait method declaration: final trait methods, __TRAIT__ context, references/copy-on-write, and native lowering remain unsupported",
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
fn emit_ir_rejects_trait_methods_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\ntrait Reusable {\n    public function render() {}\n}\n",
    )
    .unwrap_err();

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
            "unsupported null coalescing assignment: only direct variable, direct or nested array-offset, direct object-property, static-property, and direct object-property array-offset targets are implemented"
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
            "<?php\n$items = [];\necho ($items[][] = 'value');\n",
            3,
            18,
            "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented",
        ),
        (
            "<?php\n$items = [];\necho ($items[] ??= 'value');\n",
            3,
            16,
            "unsupported null coalescing assignment: only direct variable, direct or nested array-offset, direct object-property, static-property, and direct object-property array-offset targets are implemented",
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
fn emit_ir_rejects_non_direct_assignment_expression_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$items = 1;\necho ($items[0] = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_lowers_direct_reference_assignment_boundary() {
    let ir = php_compiler::emit_ir_source("<?php\n$value = 1;\n$alias =& $value;\n").unwrap();

    assert!(
        ir.contains("@phpc_native_reference_from_value_and_free")
            && ir.contains("@phpc_native_reference_clone"),
        "{ir}"
    );
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
    let error = runtime_error(
        "<?php\n$items = ['outer' => ['inner' => 1]];\necho ($items['outer']['inner'] += 2);\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "unsupported call compound assignment: nested array targets are not implemented"
    );
}

#[test]
fn emit_ir_rejects_compound_assignment_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn unsupported_increment_decrement_operators_have_stable_parse_and_runtime_errors() {
    let runtime_cases = [
        ("<?php\n$values = [[1]];\n++$values[0][0];\n", 3, 1),
        ("<?php\n$values = [[1]];\n$values[0][0]--;\n", 3, 1),
        ("<?php\n$values = [[1]];\necho ++$values[0][0];\n", 3, 6),
        ("<?php\n$values = [[1]];\necho $values[0][0]--;\n", 3, 6),
    ];

    for (source, line, column) in runtime_cases {
        let error = runtime_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported call increment/decrement: nested array targets are not implemented"
        );
    }

    let chained = parse_error("<?php\n$value = 1;\necho ++$value++;\n");
    assert_eq!(chained.line, 3);
    assert_eq!(chained.column, 6);
    assert_eq!(
        chained.message,
        "unsupported increment/decrement expression: chained increment/decrement expressions are not implemented"
    );

    let parenthesized = parse_error("<?php\n$value = 1;\n(++$value)++;\n");
    assert_eq!(parenthesized.line, 3);
    assert_eq!(parenthesized.column, 2);
    assert_eq!(
        parenthesized.message,
        "unsupported increment/decrement target: only direct static variables, direct array/object offsets, append offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer, float, string, and append-null values; append suffixes and nested variable targets are not implemented"
    );
}

#[test]
fn unsupported_for_header_increment_decrement_targets_have_stable_runtime_errors() {
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
        let error = runtime_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported call increment/decrement: nested array targets are not implemented"
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
fn foreach_destructuring_executes_and_remaining_forms_keep_parse_boundaries() {
    let destructuring_cases = [
        r#"<?php
$items = [[1]];
foreach ($items as [$item]) {
    echo $item;
}
"#,
        r#"<?php
$items = [[1]];
foreach ($items as $key => [$item]) {
    echo $item;
}
"#,
    ];

    for source in destructuring_cases {
        let execution = run_source(source).unwrap();
        assert_eq!(execution.stdout, "1");
        assert_eq!(execution.stderr, "");
        assert_eq!(execution.exit_code, 0);
    }

    let cases = [
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
