use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{AssignmentOp, BinaryOp, CastKind, Expr, Statement, UnaryOp};
use ptn::lexer::{self, TokenKind};
use ptn::{compile_file, parser, CompileOptions};

#[test]
fn parser_preserves_echo_expression_order() {
    let program = parser::parse("<?php echo \"a\", 12, true, false, null;").unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn parser_accepts_direct_assignment_and_variable_reads() {
    let program = parser::parse("<?php $greeting = \"hi\"; echo $greeting;").unwrap();
    assert_eq!(program.statements.len(), 2);
}

#[test]
fn parser_accepts_precedence_aware_binary_expressions() {
    let program = parser::parse("<?php echo \"sum \" . 2 + 3 . \"\\n\";").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::Binary {
        op: BinaryOp::Concat,
        left,
        right,
        ..
    } = &expressions[0]
    else {
        panic!("expected outer concat");
    };
    assert!(matches!(right.as_ref(), Expr::String(_, _)));

    let Expr::Binary {
        op: BinaryOp::Concat,
        right,
        ..
    } = left.as_ref()
    else {
        panic!("expected left concat");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn parser_accepts_print_as_statement() {
    let program = parser::parse("<?php print \"hello\" . 2 + 3;").unwrap();
    assert_eq!(program.statements.len(), 1);
    let Statement::Print { expression, .. } = &program.statements[0] else {
        panic!("expected print statement");
    };
    assert!(matches!(
        expression,
        Expr::Binary {
            op: BinaryOp::Concat,
            ..
        }
    ));
}

#[test]
fn parser_accepts_direct_variable_compound_assignments() {
    let program = parser::parse("<?php $value = 1; $value += 2; $value .= \"3\";").unwrap();
    assert_eq!(program.statements.len(), 3);

    let Statement::Assign { op, .. } = &program.statements[1] else {
        panic!("expected add assignment statement");
    };
    assert_eq!(*op, AssignmentOp::AddAssign);

    let Statement::Assign { op, .. } = &program.statements[2] else {
        panic!("expected concat assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ConcatAssign);
}

#[test]
fn parser_rejects_print_expression_contexts() {
    let error = parser::parse("<?php $result = print \"hello\";").unwrap_err();
    assert!(error.message.contains("expected expression"));

    let error = parser::parse("<?php $result = print(\"hello\");").unwrap_err();
    assert!(error.message.contains("expected expression"));
}

#[test]
fn parser_rejects_unsupported_increment_and_decrement_operators() {
    let increment = parser::parse("<?php echo ++$value;").unwrap_err();
    assert!(increment.message.contains("unsupported increment operator"));

    let decrement = parser::parse("<?php echo --$value;").unwrap_err();
    assert!(decrement.message.contains("unsupported decrement operator"));
}

#[test]
fn lexer_skips_php_comments_and_preserves_following_span() {
    let tokens =
        lexer::lex("<?php\n// first\n# second\n/* block\ncomment */\nprint \"ok\";").unwrap();
    let print = tokens
        .iter()
        .find(|token| matches!(token.kind, TokenKind::Print))
        .expect("expected print token");

    assert_eq!(print.span.line, 6);
    assert_eq!(print.span.column, 1);
}

#[test]
fn parser_accepts_shebang_comments_and_trailing_close_tag() {
    let program = parser::parse(
        "#!/usr/bin/env php\n<?php\n// prepare\n$name = \"PTN\";\n# emit\n/* done */\nprint $name\n?>\n",
    )
    .unwrap();

    assert_eq!(program.statements.len(), 2);
}

#[test]
fn parser_rejects_inline_html_before_open_tag() {
    let error = parser::parse("# not a shebang\n<?php print \"ok\";").unwrap_err();
    assert!(error.message.contains("expected <?php open tag"));
}

#[test]
fn parser_rejects_inline_html_after_close_tag() {
    let error = parser::parse("<?php print \"ok\"; ?> html").unwrap_err();
    assert!(error.message.contains("inline HTML after close tag"));
}

#[test]
fn parser_accepts_parenthesized_unary_and_cast_expressions() {
    let program =
        parser::parse("<?php echo -(2 + 3), !(\"0\"), (int)\"42\", (string)true;").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    let Expr::Unary {
        op: UnaryOp::Negate,
        expr,
        ..
    } = &expressions[0]
    else {
        panic!("expected unary negation");
    };
    let Expr::Grouped { expr, .. } = expr.as_ref() else {
        panic!("expected grouped negation operand");
    };
    assert!(matches!(
        expr.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));

    assert!(matches!(
        &expressions[1],
        Expr::Unary {
            op: UnaryOp::Not,
            ..
        }
    ));
    assert!(matches!(
        &expressions[2],
        Expr::Cast {
            kind: CastKind::Int,
            ..
        }
    ));
    assert!(matches!(
        &expressions[3],
        Expr::Cast {
            kind: CastKind::String,
            ..
        }
    ));
}

#[test]
fn parser_preserves_parenthesized_expression_grouping() {
    let program = parser::parse("<?php echo (1), ($name), (1 + 2), ((\"a\" . \"b\"));").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    assert!(matches!(
        &expressions[0],
        Expr::Grouped { expr, .. } if matches!(expr.as_ref(), Expr::Int(1, _))
    ));
    assert!(matches!(
        &expressions[1],
        Expr::Grouped { expr, .. } if matches!(expr.as_ref(), Expr::Variable(name, _) if name == "name")
    ));
    assert!(matches!(
        &expressions[2],
        Expr::Grouped { expr, .. } if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Add, .. })
    ));

    let Expr::Grouped { expr, .. } = &expressions[3] else {
        panic!("expected outer grouping");
    };
    assert!(matches!(
        expr.as_ref(),
        Expr::Grouped { expr, .. } if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Concat, .. })
    ));
}

#[test]
fn parser_accepts_comparison_boolean_and_grouping_expressions() {
    let program = parser::parse("<?php echo 1 + 2 <= \"3\" && (false || 4 >= 4);").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::Binary {
        op: BinaryOp::And,
        left,
        right,
        ..
    } = &expressions[0]
    else {
        panic!("expected outer boolean and");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::LessEqual,
            ..
        }
    ));
    let Expr::Grouped { expr, .. } = right.as_ref() else {
        panic!("expected grouped boolean or");
    };
    assert!(matches!(
        expr.as_ref(),
        Expr::Binary {
            op: BinaryOp::Or,
            ..
        }
    ));
}

#[test]
fn compile_echo_program_to_native_binary() {
    let root = temp_dir("ptn-native-echo");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("hello.php");
    let output = root.join("hello-bin");
    fs::write(&input, "<?php echo \"Hello \", 42, \"\\n\";").unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();
    assert!(compiled.binary.exists());
    assert!(compiled.c_source.unwrap().exists());

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "Hello 42\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_parenthesized_expression_grouping_to_native_binary() {
    let root = temp_dir("ptn-native-parenthesized-grouping");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("parenthesized-grouping.php");
    let output = root.join("parenthesized-grouping-bin");
    fs::write(
        &input,
        "<?php $name = \"Ada\"; $pair = (\"c\" . \"d\"); echo (\"lit\"), \" \", ($name), \" \", (1 + 2), \" \", ((\"a\" . \"b\")), \" \", $pair, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "lit Ada 3 ab cd\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_comments_shebang_and_trailing_close_tag_to_native_binary() {
    let root = temp_dir("ptn-native-comments-tags-after-print");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comments-tags.php");
    let output = root.join("comments-tags-bin");
    fs::write(
        &input,
        "#!/usr/bin/env php\n<?php\n// line comment\n$name = \"PTN\";\n# hash comment\n/* block\ncomment */\necho \"Hello \"; print $name . \"\\n\"\n?>\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "Hello PTN\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_print_literals_to_native_binary() {
    let root = temp_dir("ptn-native-print-literals");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("print-literals.php");
    let output = root.join("print-literals-bin");
    fs::write(&input, "<?php print \"Hello \"; print 42; print \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "Hello 42\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_print_variables_to_native_binary() {
    let root = temp_dir("ptn-native-print-variables");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("print-variables.php");
    let output = root.join("print-variables-bin");
    fs::write(
        &input,
        "<?php $name = \"PTN\"; $count = 2; print $name; print \" \"; print $count; print \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "PTN 2\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_print_binary_expression_to_native_binary() {
    let root = temp_dir("ptn-native-print-binary-expression");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("print-binary-expression.php");
    let output = root.join("print-binary-expression-bin");
    fs::write(
        &input,
        "<?php $name = \"Ada\"; print \"Hello \" . $name . \" \" . 2 + 3 . \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Hello Ada 5\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_comparisons_to_native_binary() {
    let root = temp_dir("ptn-native-comparisons");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comparisons.php");
    let output = root.join("comparisons-bin");
    fs::write(
        &input,
        "<?php echo 1 == 1, 1 != 2, 1 < 2, 2 <= 2, 3 > 2, 3 >= 3, \"42\" == \"000042\", 42 == \"42.0\", \"a\" <= \"b\", \"b\" >= \"b\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1111111111\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_less_equal_greater_equal_edges_to_native_binary() {
    let root = temp_dir("ptn-native-comparison-equality-bounds");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comparison-equality-bounds.php");
    let output = root.join("comparison-equality-bounds-bin");
    fs::write(
        &input,
        "<?php echo 2 <= \"2.0\", 3 <= 2, \"b\" >= \"a\", \"a\" >= \"b\", null <= 0, true >= \"1\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1111\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_loose_scalar_comparison_edges_to_native_binary() {
    let root = temp_dir("ptn-native-comparison-edges");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comparison-edges.php");
    let output = root.join("comparison-edges-bin");
    fs::write(
        &input,
        "<?php echo null == 0, null == \"\", null == \"0\", \"|\", 0 == \"foo\", \"|\", 2 < \"a\", \"|\", false == \"0\", true == \"0\", false == \"\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "11||1|11\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_boolean_short_circuit_ops_to_native_binary() {
    let root = temp_dir("ptn-native-boolean-short-circuit");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("short-circuit.php");
    let output = root.join("short-circuit-bin");
    fs::write(
        &input,
        "<?php echo false && $missing, \"|\", true || $missing, \"|\", true && \"0\", \"|\", false || \"non-empty\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "|1||1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_comparison_operands_evaluate_left_to_right_to_native_binary() {
    let root = temp_dir("ptn-native-comparison-left-to-right");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comparison-left-to-right.php");
    let output = root.join("comparison-left-to-right-bin");
    fs::write(&input, "<?php echo $left == $right, \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $left\nWarning: Undefined variable $right\n"
    );
}

#[test]
fn compile_direct_variable_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-variables");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("variables.php");
    let output = root.join("variables-bin");
    fs::write(
        &input,
        "<?php $name = \"PTN\"; $count = 2; echo $name, \" \", $count, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "PTN 2\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_variable_overwrite_to_native_binary() {
    let root = temp_dir("ptn-native-variable-overwrite");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("overwrite.php");
    let output = root.join("overwrite-bin");
    fs::write(
        &input,
        "<?php $value = \"old\"; $value = \"new\"; echo $value;",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "new");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_direct_compound_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-compound-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("compound.php");
    let output = root.join("compound-bin");
    fs::write(
        &input,
        "<?php $total = 1; $total += 2 + 3; $name = \"Ada\"; $name .= \" Lovelace\"; print $name . \" \" . $total . \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Ada Lovelace 6\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_compound_assignments_with_grouping_and_casts_to_native_binary() {
    let root = temp_dir("ptn-native-compound-assignment-grouped");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("compound-grouped.php");
    let output = root.join("compound-grouped-bin");
    fs::write(
        &input,
        "<?php $total = 10; $total += -(2 + (int)\"3\"); $text = \"value=\"; $text .= (string)$total; print $text . \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "value=5\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compound_assignments_read_left_before_rhs_and_then_write() {
    let root = temp_dir("ptn-native-compound-assignment-order");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("compound-order.php");
    let output = root.join("compound-order-bin");
    fs::write(
        &input,
        "<?php $total += $missing_number; print $total . \"\\n\"; $text .= $missing_text; print \"[\" . $text . \"]\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0\n[]\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $total\nWarning: Undefined variable $missing_number\nWarning: Undefined variable $text\nWarning: Undefined variable $missing_text\n"
    );
}

#[test]
fn compile_boxed_binary_operations_to_native_binary() {
    let root = temp_dir("ptn-native-binops");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("binops.php");
    let output = root.join("binops-bin");
    fs::write(
        &input,
        "<?php $name = \"Ada\"; $greeting = \"Hello \" . $name; $total = 2 + 3; echo $greeting . \" \" . $total . \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Hello Ada 5\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_numeric_string_and_float_addition_to_native_binary() {
    let root = temp_dir("ptn-native-addition-conversions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("addition-conversions.php");
    let output = root.join("addition-conversions-bin");
    fs::write(&input, "<?php echo \"2\" + 3, \" \", 1.5 + true, \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "5 2.5\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_unary_parenthesized_and_cast_expressions_to_native_binary() {
    let root = temp_dir("ptn-native-unary-casts");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unary-casts.php");
    let output = root.join("unary-casts-bin");
    fs::write(
        &input,
        "<?php echo -(2 + 3), \"\\n\"; echo !(\"0\"), \" \", !(\"x\"), \"\\n\"; echo (int)\"42\" + (float)\"0.5\", \" \", (string)true . (bool)\"0\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "-5\n1 \n42.5 1\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_binary_operands_evaluate_left_to_right_to_native_binary() {
    let root = temp_dir("ptn-native-binops-left-to-right");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("left-to-right.php");
    let output = root.join("left-to-right-bin");
    fs::write(&input, "<?php echo $left . $right . \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $left\nWarning: Undefined variable $right\n"
    );
}

#[test]
fn compile_defined_and_undefined_variable_reads_to_native_binary() {
    let root = temp_dir("ptn-native-undefined-variable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("undefined.php");
    let output = root.join("undefined-bin");
    fs::write(
        &input,
        "<?php $defined = \"defined\"; echo $defined, $missing, \" done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "defined done\n"
    );
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $missing\n"
    );
}

#[test]
fn unsupported_constructs_fail_before_codegen() {
    let unsupported_operator = parser::parse("<?php $name -= 1;").unwrap_err();
    assert!(unsupported_operator.message.contains("expected assignment"));

    let unsupported_lvalue = parser::parse("<?php $items[0] += 1;").unwrap_err();
    assert!(unsupported_lvalue
        .message
        .contains("unsupported PHP token '['"));
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
