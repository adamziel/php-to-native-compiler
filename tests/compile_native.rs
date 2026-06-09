use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{AssignmentOp, BinaryOp, CastKind, Expr, IncDecOp, Statement, StringPart, UnaryOp};
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
fn lexer_accepts_numeric_literal_separators_and_radices() {
    let tokens = lexer::lex(
        "<?php 299_792_458 96_485.332_12 6.626_070_15e-34 0xCAFE_F00D 0b0101_1111 0137_041 0_124",
    )
    .unwrap();
    assert!(matches!(tokens[1].kind, TokenKind::Int(299_792_458)));
    assert!(matches!(
        tokens[2].kind,
        TokenKind::Float(value) if value == 96_485.332_12
    ));
    assert!(matches!(
        tokens[3].kind,
        TokenKind::Float(value) if value == 6.626_070_15e-34
    ));
    assert!(matches!(tokens[4].kind, TokenKind::Int(0xCAFE_F00D)));
    assert!(matches!(tokens[5].kind, TokenKind::Int(0b0101_1111)));
    assert!(matches!(tokens[6].kind, TokenKind::Int(48_673)));
    assert!(matches!(tokens[7].kind, TokenKind::Int(84)));
}

#[test]
fn parser_accepts_precedence_aware_binary_expressions() {
    let program = parser::parse("<?php echo \"sum \" . 20 - 3 * 4 + 8 / 2 % 3 . \"\\n\";").unwrap();
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

    let Expr::Binary {
        op: BinaryOp::Add,
        right,
        ..
    } = right.as_ref()
    else {
        panic!("expected nested addition");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Modulo,
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
    let program = parser::parse(
        "<?php $value = 1; $value += 2; $value -= 3; $value *= 4; $value /= 5; $value %= 6; $value .= \"7\"; $value &= \"8\"; $value |= \"9\"; $value ^= \"10\"; $value <<= 11; $value >>= 12;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 12);

    let Statement::Assign { op, .. } = &program.statements[1] else {
        panic!("expected add assignment statement");
    };
    assert_eq!(*op, AssignmentOp::AddAssign);

    let Statement::Assign { op, .. } = &program.statements[2] else {
        panic!("expected subtract assignment statement");
    };
    assert_eq!(*op, AssignmentOp::SubtractAssign);

    let Statement::Assign { op, .. } = &program.statements[3] else {
        panic!("expected multiply assignment statement");
    };
    assert_eq!(*op, AssignmentOp::MultiplyAssign);

    let Statement::Assign { op, .. } = &program.statements[4] else {
        panic!("expected divide assignment statement");
    };
    assert_eq!(*op, AssignmentOp::DivideAssign);

    let Statement::Assign { op, .. } = &program.statements[5] else {
        panic!("expected modulo assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ModuloAssign);

    let Statement::Assign { op, .. } = &program.statements[6] else {
        panic!("expected concat assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ConcatAssign);

    let Statement::Assign { op, .. } = &program.statements[7] else {
        panic!("expected bitwise and assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseAndAssign);

    let Statement::Assign { op, .. } = &program.statements[8] else {
        panic!("expected bitwise or assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseOrAssign);

    let Statement::Assign { op, .. } = &program.statements[9] else {
        panic!("expected bitwise xor assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseXorAssign);

    let Statement::Assign { op, .. } = &program.statements[10] else {
        panic!("expected shift left assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ShiftLeftAssign);

    let Statement::Assign { op, .. } = &program.statements[11] else {
        panic!("expected shift right assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ShiftRightAssign);
}

#[test]
fn parser_accepts_direct_variable_increment_decrement_statements() {
    let program = parser::parse("<?php $value = 1; $value++; ++$value; $value--; --$value; while ($value < 3) { $value++; }").unwrap();
    assert_eq!(program.statements.len(), 6);

    let Statement::Increment { op, .. } = &program.statements[1] else {
        panic!("expected postfix increment statement");
    };
    assert_eq!(*op, IncDecOp::Increment);

    let Statement::Increment { op, .. } = &program.statements[4] else {
        panic!("expected prefix decrement statement");
    };
    assert_eq!(*op, IncDecOp::Decrement);

    let Statement::While { body, .. } = &program.statements[5] else {
        panic!("expected while statement");
    };
    assert!(matches!(
        &body[0],
        Statement::Increment {
            op: IncDecOp::Increment,
            ..
        }
    ));
}

#[test]
fn parser_accepts_braced_do_while_statements() {
    let program = parser::parse("<?php $i = 3; do { echo $i; $i--; } while ($i > 0);").unwrap();
    let Statement::DoWhile {
        body, condition, ..
    } = &program.statements[1]
    else {
        panic!("expected do-while statement");
    };
    assert_eq!(body.len(), 2);
    assert!(matches!(
        condition,
        Expr::Binary {
            op: BinaryOp::Greater,
            ..
        }
    ));
}

#[test]
fn parser_accepts_braced_for_statements() {
    let program = parser::parse("<?php for ($i = 0; $i < 3; ++$i) { echo $i; }").unwrap();
    let Statement::For {
        initializers,
        condition,
        updates,
        body,
        ..
    } = &program.statements[0]
    else {
        panic!("expected for statement");
    };
    assert_eq!(initializers.len(), 1);
    assert!(matches!(
        &initializers[0],
        Statement::Assign {
            op: AssignmentOp::Assign,
            ..
        }
    ));
    assert!(matches!(
        condition,
        Some(Expr::Binary {
            op: BinaryOp::Less,
            ..
        })
    ));
    assert!(matches!(
        &updates[0],
        Statement::Increment {
            op: IncDecOp::Increment,
            ..
        }
    ));
    assert_eq!(body.len(), 1);
}

#[test]
fn parser_rejects_print_expression_contexts() {
    let error = parser::parse("<?php $result = print \"hello\";").unwrap_err();
    assert!(error.message.contains("expected expression"));

    let error = parser::parse("<?php $result = print(\"hello\");").unwrap_err();
    assert!(error.message.contains("expected expression"));
}

#[test]
fn parser_rejects_increment_and_decrement_expression_contexts() {
    let increment = parser::parse("<?php echo ++$value;").unwrap_err();
    assert!(increment.message.contains("expected expression"));

    let decrement = parser::parse("<?php echo $value--;").unwrap_err();
    assert!(decrement.message.contains("expected semicolon"));

    let invalid_prefix = parser::parse("<?php ++1;").unwrap_err();
    assert!(invalid_prefix.message.contains("expected variable"));
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
fn parser_accepts_inline_html_after_close_tag_as_output() {
    let program = parser::parse("<?php print \"ok\"; ?> html").unwrap();
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(
        &program.statements[1],
        Statement::InlineHtml { content, .. } if content == " html"
    ));
}

#[test]
fn parser_rejects_inline_html_between_php_blocks() {
    let error = parser::parse("<?php print \"ok\"; ?> html <?php print \"again\";").unwrap_err();
    assert!(error.message.contains("inline HTML between PHP blocks"));
}

#[test]
fn parser_accepts_internal_call_statements_and_inline_html() {
    let program = parser::parse("<?php VAR_DUMP(null, true, 2 < 3); ?>\nDONE\n").unwrap();
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(
        &program.statements[0],
        Statement::Call { name, arguments, .. } if name == "var_dump" && arguments.len() == 3
    ));
    assert!(matches!(
        &program.statements[1],
        Statement::InlineHtml { content, .. } if content == "DONE\n"
    ));
}

#[test]
fn parser_accepts_internal_call_expressions() {
    let program = parser::parse("<?php echo strlen(\"abc\"), strlen((string)42) + 1;").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Call { name, arguments, .. } if name == "strlen" && arguments.len() == 1
    ));
    assert!(matches!(
        &expressions[1],
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn parser_accepts_parenthesized_unary_and_cast_expressions() {
    let program =
        parser::parse("<?php echo +(2 + 3), -(2 + 3), !(\"0\"), (int)\"42\", (string)true;")
            .unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    let Expr::Unary {
        op: UnaryOp::Positive,
        expr,
        ..
    } = &expressions[0]
    else {
        panic!("expected unary plus");
    };
    let Expr::Grouped { expr, .. } = expr.as_ref() else {
        panic!("expected grouped unary plus operand");
    };
    assert!(matches!(
        expr.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));

    let Expr::Unary {
        op: UnaryOp::Negate,
        expr,
        ..
    } = &expressions[1]
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
        &expressions[2],
        Expr::Unary {
            op: UnaryOp::Not,
            ..
        }
    ));
    assert!(matches!(
        &expressions[3],
        Expr::Cast {
            kind: CastKind::Int,
            ..
        }
    ));
    assert!(matches!(
        &expressions[4],
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
fn parser_accepts_strict_identity_expressions() {
    let program = parser::parse("<?php echo 1 === 1, \"1\" !== 1;").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Binary {
            op: BinaryOp::Identical,
            ..
        }
    ));
    assert!(matches!(
        &expressions[1],
        Expr::Binary {
            op: BinaryOp::NotIdentical,
            ..
        }
    ));
}

#[test]
fn parser_accepts_bitwise_scalar_expressions() {
    let program = parser::parse("<?php echo \"a\" & \"b\" ^ \"d\" | \"c\" && 1 == 1;").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    let Expr::Binary {
        op: BinaryOp::And,
        left,
        ..
    } = &expressions[0]
    else {
        panic!("expected outer boolean and");
    };
    let Expr::Binary {
        op: BinaryOp::BitwiseOr,
        left,
        ..
    } = left.as_ref()
    else {
        panic!("expected bitwise or below boolean and");
    };
    let Expr::Binary {
        op: BinaryOp::BitwiseXor,
        left,
        ..
    } = left.as_ref()
    else {
        panic!("expected bitwise xor below bitwise or");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::BitwiseAnd,
            ..
        }
    ));
}

#[test]
fn parser_accepts_unary_bitwise_not_expressions() {
    let program = parser::parse("<?php echo ~6 & 3, ~(\"some\");").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::Binary {
        op: BinaryOp::BitwiseAnd,
        left,
        ..
    } = &expressions[0]
    else {
        panic!("expected bitwise and expression");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Unary {
            op: UnaryOp::BitwiseNot,
            ..
        }
    ));
    assert!(matches!(
        &expressions[1],
        Expr::Unary {
            op: UnaryOp::BitwiseNot,
            ..
        }
    ));
}

#[test]
fn parser_accepts_shift_expressions_and_bare_constants() {
    let program = parser::parse(
        "<?php error_reporting(\\E_ERROR); var_dump(\"34\" << \"1\", \"56\" >> \"2\", \\PHP_EOL);",
    )
    .unwrap();

    let Statement::Call {
        name, arguments, ..
    } = &program.statements[0]
    else {
        panic!("expected error_reporting call");
    };
    assert_eq!(name, "error_reporting");
    assert!(matches!(
        &arguments[0],
        Expr::Constant(constant, _) if constant == "E_ERROR"
    ));

    let Statement::Call { arguments, .. } = &program.statements[1] else {
        panic!("expected var_dump call");
    };
    assert!(matches!(
        &arguments[0],
        Expr::Binary {
            op: BinaryOp::ShiftLeft,
            ..
        }
    ));
    assert!(matches!(
        &arguments[1],
        Expr::Binary {
            op: BinaryOp::ShiftRight,
            ..
        }
    ));
    assert!(matches!(
        &arguments[2],
        Expr::Constant(constant, _) if constant == "PHP_EOL"
    ));
}

#[test]
fn parser_accepts_braced_if_elseif_else_statements() {
    let program = parser::parse(
        "<?php $a = 1; if (($a == 0)) { echo \"bad\"; } elseif ($a == 1) { var_dump(true); } else { print \"bad\"; }",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 2);
    let Statement::If {
        condition,
        then_body,
        else_body,
        ..
    } = &program.statements[1]
    else {
        panic!("expected if statement");
    };
    assert!(matches!(condition, Expr::Grouped { .. }));
    assert_eq!(then_body.len(), 1);
    assert_eq!(else_body.len(), 1);
    assert!(matches!(&else_body[0], Statement::If { .. }));
}

#[test]
fn parser_accepts_braced_switch_cases_default_and_break() {
    let program = parser::parse(
        "<?php $a = 1; switch ($a) { case 0: echo \"bad\"; break; case 1: echo \"good\"; break; default: echo \"bad\"; break; }",
    )
    .unwrap();

    let Statement::Switch {
        expression, cases, ..
    } = &program.statements[1]
    else {
        panic!("expected switch statement");
    };
    assert!(matches!(expression, Expr::Variable(name, _) if name == "a"));
    assert_eq!(cases.len(), 3);
    assert!(matches!(cases[0].condition, Some(Expr::Int(0, _))));
    assert!(matches!(cases[1].condition, Some(Expr::Int(1, _))));
    assert!(cases[2].condition.is_none());
    assert!(matches!(
        cases[1].body.last(),
        Some(Statement::Break { .. })
    ));
}

#[test]
fn parser_accepts_direct_variable_interpolated_strings() {
    let program = parser::parse("<?php echo \"value=$value\\n\", \"literal\\$value\\n\";").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::InterpolatedString(parts, _) = &expressions[0] else {
        panic!("expected interpolated string");
    };
    assert_eq!(
        parts,
        &vec![
            StringPart::Literal("value=".to_string()),
            StringPart::Variable("value".to_string()),
            StringPart::Literal("\n".to_string()),
        ]
    );
    assert!(matches!(&expressions[1], Expr::String(value, _) if value == "literal$value\n"));
}

#[test]
fn parser_rejects_multiple_switch_defaults() {
    let error =
        parser::parse("<?php switch (1) { default: echo 1; break; default: echo 2; break; }")
            .unwrap_err();
    assert_eq!(
        error.message,
        "Switch statements may only contain one default clause"
    );
    assert_eq!(error.span.unwrap().line, 1);
}

#[test]
fn phpc_renders_spanned_compile_diagnostics_as_php_fatals() {
    let root = temp_dir("ptn-phpc-source-fatal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("duplicate-default.php");
    fs::write(
        &input,
        "<?php\n\nswitch (1) {\n    default:\n        print 1;\n    default:\n        print 2;\n}\n",
    )
    .unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Switch statements may only contain one default clause in {} on line 6\n",
            input.display()
        )
    );
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
fn compile_var_dump_scalars_to_native_binary() {
    let root = temp_dir("ptn-native-var-dump-scalars");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-dump-scalars.php");
    let output = root.join("var-dump-scalars-bin");
    fs::write(
        &input,
        "<?php var_dump(null, true, false, (int)\"42\", -(1.5 + 0.5), (string)true, 2 < 3);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "NULL\nbool(true)\nbool(false)\nint(42)\nfloat(-2)\nstring(1) \"1\"\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_var_dump_null_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-var-dump-null-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-dump-null-phpt-shape.php");
    let output = root.join("var-dump-null-phpt-shape-bin");
    fs::write(&input, "<?php\n\nvar_dump(null);\n\n?>\nDONE\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "NULL\nDONE\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_internal_call_arguments_evaluate_left_to_right_to_native_binary() {
    let root = temp_dir("ptn-native-internal-call-left-to-right");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("internal-call-left-to-right.php");
    let output = root.join("internal-call-left-to-right-bin");
    fs::write(&input, "<?php var_dump($left, $right);").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "NULL\nNULL\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $left\nWarning: Undefined variable $right\n"
    );
}

#[test]
fn compile_strlen_expression_to_native_binary() {
    let root = temp_dir("ptn-native-strlen-expression");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strlen-expression.php");
    let output = root.join("strlen-expression-bin");
    fs::write(
        &input,
        "<?php echo strlen(\"abcdef\"), \" \", strlen((string)42) + 1, \" \", strlen(false), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "6 3 0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_str_rot13_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-str-rot13-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-rot13-basic.php");
    let output = root.join("str-rot13-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing str_rot13() : basic functionality ***\\n\";\n\
\n\
echo \"\\nBasic tests\\n\";\n\
var_dump(str_rot13(\"str_rot13() tests starting\"));\n\
var_dump(str_rot13(\"abcdefghijklmnopqrstuvwxyz\"));\n\
\n\
echo \"\\nEnsure numeric characters are left untouched\\n\";\n\
if (strcmp(str_rot13(\"0123456789\"), \"0123456789\") == 0) {\n\
    echo \"Strings equal : TEST PASSED\\n\";\n\
} else {\n\
    echo \"Strings unequal : TEST FAILED\\n\";\n\
}\n\
\n\
echo \"\\nEnsure non-alphabetic characters are left untouched\\n\";\n\
if (strcmp(str_rot13(\"!%^&*()_-+={}[]:;@~#<,>.?\"), \"!%^&*()_-+={}[]:;@~#<,>.?\")) {\n\
    echo \"Strings equal : TEST PASSED\\n\";\n\
} else {\n\
    echo \"Strings unequal : TEST FAILED\\n\";\n\
}\n\
\n\
echo \"\\nEnsure strings round trip\\n\";\n\
$str = \"str_rot13() tests starting\";\n\
$encode = str_rot13($str);\n\
$decode = str_rot13($encode);\n\
if (strcmp($str, $decode) == 0) {\n\
    echo \"Strings equal : TEST PASSED\\n\";\n\
} else {\n\
    echo \"Strings unequal : TEST FAILED\\n\";\n\
}\n\
var_dump(function_exists(\"str_rot13\"), function_exists(\"STRCMP\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing str_rot13() : basic functionality ***\n\nBasic tests\nstring(26) \"fge_ebg13() grfgf fgnegvat\"\nstring(26) \"nopqrstuvwxyzabcdefghijklm\"\n\nEnsure numeric characters are left untouched\nStrings equal : TEST PASSED\n\nEnsure non-alphabetic characters are left untouched\nStrings unequal : TEST FAILED\n\nEnsure strings round trip\nStrings equal : TEST PASSED\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_internal_call_expression_arguments_evaluate_left_to_right() {
    let root = temp_dir("ptn-native-call-expression-left-to-right");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("call-expression-left-to-right.php");
    let output = root.join("call-expression-left-to-right-bin");
    fs::write(&input, "<?php echo strlen($left . $right), \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $left\nWarning: Undefined variable $right\n"
    );
}

#[test]
fn compile_statement_call_discards_internal_return_value() {
    let root = temp_dir("ptn-native-call-statement-discard");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("call-statement-discard.php");
    let output = root.join("call-statement-discard-bin");
    fs::write(&input, "<?php strlen(\"abcdef\"); echo \"done\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "done\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_type_internal_functions_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-type-functions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-type-functions.php");
    let output = root.join("scalar-type-functions-bin");
    fs::write(
        &input,
        "<?php var_dump(gettype(null), gettype(true), gettype(42), gettype(1.5), gettype(\"x\")); var_dump(is_null(null), is_bool(false), is_int(1), is_integer(1), is_long(1), is_float(1.5), is_double(1.5), is_string(\"x\"), is_scalar(\"x\"), is_scalar(null), is_float('-.1' * 2));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"NULL\"\nstring(7) \"boolean\"\nstring(7) \"integer\"\nstring(6) \"double\"\nstring(6) \"string\"\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_finite_infinite_nan_internal_functions_to_native_binary() {
    let root = temp_dir("ptn-native-finite-infinite-nan-functions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("finite-infinite-nan-functions.php");
    let output = root.join("finite-infinite-nan-functions-bin");
    fs::write(
        &input,
        "<?php var_dump(is_finite(INF)); var_dump(is_infinite(INF)); var_dump(is_nan(INF)); var_dump(is_finite(-INF)); var_dump(is_infinite(-INF)); var_dump(is_nan(-INF)); var_dump(is_finite(NAN)); var_dump(is_infinite(NAN)); var_dump(is_nan(NAN)); var_dump(function_exists(\"is_nan\"), function_exists(\"IS_INFINITE\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_bug30726_is_float_shape_to_native_binary() {
    let root = temp_dir("ptn-native-bug30726-is-float");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bug30726.php");
    let output = root.join("bug30726-bin");
    fs::write(&input, "<?php echo (int) is_float('-.1' * 2), \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-chr-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-function.php");
    let output = root.join("chr-function-bin");
    fs::write(
        &input,
        "<?php echo chr(72). chr(101) . chr(108) . chr(108). chr(111); echo chr(10); echo bin2hex(chr(255)), \" \", bin2hex(chr(-1)), \" \", bin2hex(chr(\"65\")), \"\\n\"; var_dump(function_exists(\"chr\"), function_exists(\"CHR\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Hello\nff ff 41\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-chr-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-basic.php");
    let output = root.join("chr-basic-bin");
    fs::write(
        &input,
        "<?php\n\necho \"*** Testing chr() : basic functionality ***\\n\";\n\necho chr(72). chr(101) . chr(108) . chr(108). chr(111); // Hello\necho chr(10); // \"\\n\"\necho \"World\";\necho \"\\n\";\n?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing chr() : basic functionality ***\nHello\nWorld\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_ord_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-ord-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("ord-function.php");
    let output = root.join("ord-function-bin");
    fs::write(
        &input,
        "<?php echo ord(\"a\"), \" \", ord(\"9\"), \" \", ord(chr(255)), \" \", ord(true), \"\\n\"; var_dump(function_exists(\"ord\"), function_exists(\"ORD\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "97 57 255 49\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_ord_not_one_byte_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-ord-not-one-byte-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("ord-not-one-byte.php");
    let output = root.join("ord-not-one-byte-bin");
    fs::write(
        &input,
        "<?php\n\nvar_dump(ord(\"\"));\nvar_dump(ord(\"Hello\"));\n\n?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Deprecated: ord(): Providing an empty string is deprecated in ptn on line 3\nint(0)\n\nDeprecated: ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead in ptn on line 4\nint(72)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_base_conversion_internal_functions_to_native_binary() {
    let root = temp_dir("ptn-native-base-conversion-functions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("base-conversion-functions.php");
    let output = root.join("base-conversion-functions-bin");
    fs::write(
        &input,
        "<?php var_dump(bindec(\"101\"), bindec(\"0B101\"), hexdec(\"ff\"), hexdec(\"0X10\"), octdec(\"77\"), octdec(\"0O10\"), function_exists(\"hexdec\"), function_exists(\"HEXDEC\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(5)\nint(5)\nint(255)\nint(16)\nint(63)\nint(8)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_base_conversion_invalid_character_diagnostic_to_native_binary() {
    let root = temp_dir("ptn-native-base-conversion-invalid-character");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("base-conversion-invalid-character.php");
    let output = root.join("base-conversion-invalid-character-bin");
    fs::write(&input, "<?php\n\nvar_dump(hexdec(\"f?f\"));\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Deprecated: Invalid characters passed for attempted conversion, these have been ignored in ptn on line 3\nint(255)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_base_conversion_variation2_phpt_shapes_to_native_binary() {
    let root = temp_dir("ptn-native-base-conversion-variation2-phpt-shapes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("base-conversion-variation2.php");
    let output = root.join("base-conversion-variation2-bin");
    fs::write(
        &input,
        "<?php\n\nvar_dump(bindec('0b'));\nvar_dump(bindec('0B'));\nvar_dump(bindec(''));\nvar_dump(hexdec('0x'));\nvar_dump(hexdec('0X'));\nvar_dump(hexdec(''));\nvar_dump(octdec('0o'));\nvar_dump(octdec('0O'));\nvar_dump(octdec('0'));\nvar_dump(octdec(''));\n\n?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_floorceil_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-floorceil-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("floorceil.php");
    let output = root.join("floorceil-bin");
    fs::write(
        &input,
        "<?php\n    $a = ceil (-0);   $b = ceil (-1);   $c = ceil (-1.5);\n    $d = ceil (-1.8); $e = ceil (-2.7);\n    var_dump ($a, $b, $c, $d, $e);\n\n    $a = ceil (0);   $b = ceil (0.5); $c = ceil (1);\n    $d = ceil (1.5); $e = ceil (1.8); $f = ceil (2.7);\n    var_dump ($a, $b, $c, $d, $e, $f);\n\n    $a = floor (-0);   $b = floor (-0.5); $c = floor (-1);\n    $d = floor (-1.5); $e = floor (-1.8); $f = floor (-2.7);\n    var_dump ($a, $b, $c, $d, $e, $f);\n\n    $a = floor (0);   $b = floor (0.5); $c = floor (1);\n    $d = floor (1.5); $e = floor (1.8); $f = floor (2.7);\n    var_dump ($a, $b, $c, $d, $e, $f);\n    var_dump(function_exists(\"ceil\"), function_exists(\"FLOOR\"));\n?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(0)\nfloat(-1)\nfloat(-1)\nfloat(-1)\nfloat(-2)\nfloat(0)\nfloat(1)\nfloat(1)\nfloat(2)\nfloat(2)\nfloat(3)\nfloat(0)\nfloat(-1)\nfloat(-1)\nfloat(-2)\nfloat(-2)\nfloat(-3)\nfloat(0)\nfloat(0)\nfloat(1)\nfloat(1)\nfloat(1)\nfloat(2)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_pi_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-pi-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("pi-basic.php");
    let output = root.join("pi-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo pi(), \"\\n\";\n\
echo M_PI, \"\\n\";\n\
var_dump(function_exists(\"pi\"), function_exists(\"PI\"), defined(\"M_PI\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "3.1415926535898\n3.1415926535898\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_math_constants_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-math-constants-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("math-constants-basic.php");
    let output = root.join("math-constants-basic-bin");
    fs::write(
        &input,
        "<?php\necho \"M_E= \";\nvar_dump(M_E);\necho \"M_LOG2E= \";\nvar_dump(M_LOG2E);\necho \"M_LOG10E= \";\nvar_dump(M_LOG10E);\necho \"M_LN2= \";\nvar_dump(M_LN2);\necho \"M_LN10= \";\nvar_dump(M_LN10);\necho \"M_PI= \";\nvar_dump(M_PI);\necho \"M_PI_2= \";\nvar_dump(M_PI_2);\necho \"M_PI_4= \";\nvar_dump(M_PI_4);\necho \"M_1_PI= \";\nvar_dump(M_1_PI);\necho \"M_2_PI= \";\nvar_dump(M_2_PI);\necho \"M_SQRTPI= \";\nvar_dump(M_SQRTPI);\necho \"M_2_SQRTPI= \";\nvar_dump(M_2_SQRTPI);\necho \"M_LNPI= \";\nvar_dump(M_LNPI);\necho \"M_EULER= \";\nvar_dump(M_EULER);\necho \"M_SQRT2= \";\nvar_dump(M_SQRT2);\necho \"M_SQRT1_2= \";\nvar_dump(M_SQRT1_2);\necho \"M_SQRT3= \";\nvar_dump(M_SQRT3);\necho \"INF= \";\nvar_dump(INF);\necho \"NAN= \";\nvar_dump(NAN);\nvar_dump(defined(\"M_E\"), defined(\"M_SQRT3\"));\n?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "M_E= float(2.718281828459045)\nM_LOG2E= float(1.4426950408889634)\nM_LOG10E= float(0.4342944819032518)\nM_LN2= float(0.6931471805599453)\nM_LN10= float(2.302585092994046)\nM_PI= float(3.141592653589793)\nM_PI_2= float(1.5707963267948966)\nM_PI_4= float(0.7853981633974483)\nM_1_PI= float(0.3183098861837907)\nM_2_PI= float(0.6366197723675814)\nM_SQRTPI= float(1.772453850905516)\nM_2_SQRTPI= float(1.1283791670955126)\nM_LNPI= float(1.1447298858494002)\nM_EULER= float(0.5772156649015329)\nM_SQRT2= float(1.4142135623730951)\nM_SQRT1_2= float(0.7071067811865476)\nM_SQRT3= float(1.7320508075688772)\nINF= float(INF)\nNAN= float(NAN)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_sqrt_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-sqrt-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sqrt-basic.php");
    let output = root.join("sqrt-basic-bin");
    fs::write(
        &input,
        "<?php\n\
$arg_0 = 9.0;\n\
\n\
var_dump(sqrt($arg_0));\n\
var_dump(function_exists(\"sqrt\"), function_exists(\"SQRT\"));\n\
\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(3)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_getrandmax_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-getrandmax-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("getrandmax-basic.php");
    let output = root.join("getrandmax-basic-bin");
    fs::write(
        &input,
        "<?php\n\
$biggest_int = getrandmax();\n\
var_dump($biggest_int);\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(2147483647)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_getrandmax_registry_to_native_binary() {
    let root = temp_dir("ptn-native-getrandmax-registry");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("getrandmax-registry.php");
    let output = root.join("getrandmax-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"getrandmax\"), function_exists(\"GETRANDMAX\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_getmypid_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-getmypid-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("getmypid-basic.php");
    let output = root.join("getmypid-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"Simple testcase for getmypid() function\\n\";\n\
\n\
var_dump(getmypid());\n\
\n\
echo \"Done\\n\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let stdout = String::from_utf8(execution.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Simple testcase for getmypid() function");
    assert!(lines[1].starts_with("int(") && lines[1].ends_with(')'));
    let pid: i64 = lines[1][4..lines[1].len() - 1].parse().unwrap();
    assert!(pid > 0);
    assert_eq!(lines[2], "Done");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_getmypid_registry_to_native_binary() {
    let root = temp_dir("ptn-native-getmypid-registry");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("getmypid-registry.php");
    let output = root.join("getmypid-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"getmypid\"), function_exists(\"GETMYPID\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_symbol_existence_internal_functions_to_native_binary() {
    let root = temp_dir("ptn-native-symbol-existence-functions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("symbol-existence-functions.php");
    let output = root.join("symbol-existence-functions-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"strlen\"), function_exists(\"STRLEN\"), function_exists(\"sapi_windows_vt100_support\"), defined(\"test\"), defined(\"E_ERROR\")); echo gettype(defined(\"test\")), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nboolean\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_php_int_constants_to_native_binary() {
    let root = temp_dir("ptn-native-php-int-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("php-int-constants.php");
    let output = root.join("php-int-constants-bin");
    fs::write(
        &input,
        "<?php var_dump(PHP_INT_MIN, PHP_INT_MAX, PHP_INT_SIZE, defined(\"PHP_INT_MIN\"), defined(\"PHP_INT_MAX\"), defined(\"PHP_INT_SIZE\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(-9223372036854775808)\nint(9223372036854775807)\nint(8)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_directory_separator_constants_to_native_binary() {
    let root = temp_dir("ptn-native-directory-separator-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("directory-separator-constants.php");
    let output = root.join("directory-separator-constants-bin");
    fs::write(
        &input,
        "<?php\n\
echo DIRECTORY_SEPARATOR;\n\
echo \"\\n\";\n\
echo PATH_SEPARATOR;\n\
echo \"\\n\";\n\
echo \"done\\n\";\n\
var_dump(defined(\"DIRECTORY_SEPARATOR\"), defined(\"PATH_SEPARATOR\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let directory_separator = if cfg!(windows) { "\\" } else { "/" };
    let path_separator = if cfg!(windows) { ";" } else { ":" };
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!("{directory_separator}\n{path_separator}\ndone\nbool(true)\nbool(true)\n")
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_bug27443_defined_type_shape_to_native_binary() {
    let root = temp_dir("ptn-native-bug27443-defined");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bug27443.php");
    let output = root.join("bug27443-bin");
    fs::write(&input, "<?php echo gettype(defined('test'));").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "boolean");
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
fn compile_nan_scalar_comparisons_to_native_binary() {
    let root = temp_dir("ptn-native-nan-scalar-comparisons");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nan-scalar-comparisons.php");
    let output = root.join("nan-scalar-comparisons-bin");
    fs::write(
        &input,
        "<?php echo \"** CONST\\n\"; var_dump(0 < NAN); var_dump(0 <= NAN); var_dump(0 > NAN); var_dump(0 >= NAN); echo \"** VAR\\n\"; $nan = NAN; var_dump(0 < $nan); var_dump(0 <= $nan); var_dump(0 > $nan); var_dump(0 >= $nan); var_dump(NAN == NAN); var_dump(NAN != NAN); var_dump(NAN === NAN); var_dump(NAN !== NAN); var_dump(false < NAN); var_dump(true <= NAN); var_dump(null < NAN); var_dump(NAN > null); var_dump(\"0\" <= NAN); var_dump(NAN >= \"NAN\");",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "** CONST\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n** VAR\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\n"
    );
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
fn compile_strict_identity_comparisons_to_native_binary() {
    let root = temp_dir("ptn-native-strict-identity");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strict-identity.php");
    let output = root.join("strict-identity-bin");
    fs::write(
        &input,
        "<?php $negativeZero = -0.0; echo 1 === 1, 1 === \"1\", \"1\" !== 1, $negativeZero === (float)(int)$negativeZero, $negativeZero === 0.0, null === null, false !== null, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "111111\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_shift_strings_and_constants_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-shifts");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-shifts.php");
    let output = root.join("scalar-shifts-bin");
    fs::write(
        &input,
        "<?php error_reporting(E_ERROR); var_dump(\"12\" << \"0\"); var_dump(\"34\" << \"1\"); var_dump(\"56\" << \"2\"); var_dump(\"12\" >> \"0\"); var_dump(\"34\" >> \"1\"); var_dump(\"56\" >> \"2\"); var_dump(defined(\"E_ERROR\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(12)\nint(68)\nint(224)\nint(12)\nint(17)\nint(14)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_shift_compound_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-shift-compound-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("shift-compound.php");
    let output = root.join("shift-compound-bin");
    fs::write(
        &input,
        "<?php echo 'Bitwise ops:' . \\PHP_EOL; $var = 3; $var |= 1.0; var_dump($var); $var = 3; $var &= 1.0; var_dump($var); $var = 3; $var ^= 1.0; var_dump($var); $var = 3; $var <<= 1.0; var_dump($var); $var = 3; $var >>= 1.0; var_dump($var); echo 'Modulo:' . \\PHP_EOL; $var = 9; $var %= 2.0; var_dump($var);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Bitwise ops:\nint(3)\nint(1)\nint(2)\nint(6)\nint(1)\nModulo:\nint(1)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn phpc_run_alias_executes_compiled_native_binary() {
    let root = temp_dir("ptn-phpc-run-alias");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("run-alias.php");
    fs::write(&input, "<?php echo 2 === 2, \"\\n\";").unwrap();

    let execution = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .arg("run")
        .arg(&input)
        .output()
        .unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_if_elseif_else_to_native_binary() {
    let root = temp_dir("ptn-native-if-elseif-else");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("if-elseif-else.php");
    let output = root.join("if-elseif-else-bin");
    fs::write(
        &input,
        "<?php $a = 1; if ($a == 0) { echo \"bad\"; } elseif ($a == 1) { echo \"good\"; } else { echo \"bad\"; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "good\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_nested_if_branch_truthiness_to_native_binary() {
    let root = temp_dir("ptn-native-nested-if-truthiness");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-if-truthiness.php");
    let output = root.join("nested-if-truthiness-bin");
    fs::write(
        &input,
        "<?php $a = 1; $b = \"0\"; if ($a && !$b) { if ((2 >= 2)) { var_dump(\"ok\"); } else { echo \"bad\"; } } else { echo \"bad\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(2) \"ok\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_if_condition_evaluates_before_selected_branch_to_native_binary() {
    let root = temp_dir("ptn-native-if-condition-order");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("if-condition-order.php");
    let output = root.join("if-condition-order-bin");
    fs::write(
        &input,
        "<?php if ($missing) { echo \"bad\"; } else { echo \"fallback\\n\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "fallback\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $missing\n"
    );
}

#[test]
fn compile_braced_switch_to_native_binary() {
    let root = temp_dir("ptn-native-switch-basic");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("switch-basic.php");
    let output = root.join("switch-basic-bin");
    fs::write(
        &input,
        "<?php $a = 1; switch($a) { case 0: echo \"bad\"; break; case 1: echo \"good\"; break; default: echo \"bad\"; break; } ?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "good");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_switch_default_and_case_fallthrough_to_native_binary() {
    let root = temp_dir("ptn-native-switch-fallthrough");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("switch-fallthrough.php");
    let output = root.join("switch-fallthrough-bin");
    fs::write(
        &input,
        "<?php switch (\"x\") { case \"y\": echo \"bad\"; break; default: echo \"default\"; case \"z\": echo \" fall\"; break; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "default fall\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_switch_evaluates_subject_and_cases_until_match() {
    let root = temp_dir("ptn-native-switch-evaluation-order");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("switch-evaluation-order.php");
    let output = root.join("switch-evaluation-order-bin");
    fs::write(
        &input,
        "<?php switch (var_dump(\"subject\")) { case var_dump(\"case1\"): echo \"matched\\n\"; break; case var_dump(\"case2\"): echo \"bad\\n\"; break; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(7) \"subject\"\nstring(5) \"case1\"\nmatched\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_while_with_postfix_increment_to_native_binary() {
    let root = temp_dir("ptn-native-while-postfix-increment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("while-postfix-increment.php");
    let output = root.join("while-postfix-increment-bin");
    fs::write(&input, "<?php $a = 1; while ($a < 10) { echo $a; $a++; }").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "123456789");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_prefix_increment_and_decrement_to_native_binary() {
    let root = temp_dir("ptn-native-prefix-increment-decrement");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("prefix-increment-decrement.php");
    let output = root.join("prefix-increment-decrement-bin");
    fs::write(
        &input,
        "<?php $value = 1; ++$value; echo $value, \" \"; $value++; echo $value, \" \"; --$value; echo $value, \" \"; $value--; echo $value, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "2 3 2 1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_while_condition_rechecks_each_iteration_to_native_binary() {
    let root = temp_dir("ptn-native-while-condition-recheck");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("while-condition-recheck.php");
    let output = root.join("while-condition-recheck-bin");
    fs::write(
        &input,
        "<?php $enabled = true; $i = 0; while ($enabled && $i < 3) { echo $i; $i++; if ($i == 2) { $enabled = false; } } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "01\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_do_while_runs_body_before_condition_to_native_binary() {
    let root = temp_dir("ptn-native-do-while-post-test");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("do-while-post-test.php");
    let output = root.join("do-while-post-test-bin");
    fs::write(
        &input,
        "<?php $a = 0; do { echo $a; $a++; } while ($a < 0); echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_do_while_countdown_to_native_binary() {
    let root = temp_dir("ptn-native-do-while-countdown");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("do-while-countdown.php");
    let output = root.join("do-while-countdown-bin");
    fs::write(
        &input,
        "<?php $i = 3; do { echo $i; $i--; } while ($i > 0); echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "321\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_braced_for_loop_to_native_binary() {
    let root = temp_dir("ptn-native-for-loop");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("for-loop.php");
    let output = root.join("for-loop-bin");
    fs::write(
        &input,
        "<?php for ($i = 0; $i < 4; ++$i) { echo $i; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0123\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_for_break_skips_update_to_native_binary() {
    let root = temp_dir("ptn-native-for-break");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("for-break.php");
    let output = root.join("for-break-bin");
    fs::write(
        &input,
        "<?php $after = 0; for ($i = 0; $i < 5; $i++) { echo $i; break; } echo \":\", $i, \":\", $after, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0:0:0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_direct_variable_interpolation_to_native_binary() {
    let root = temp_dir("ptn-native-direct-interpolation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("direct-interpolation.php");
    let output = root.join("direct-interpolation-bin");
    fs::write(
        &input,
        "<?php $name = \"Ada\"; $count = 3; echo \"name=$name count=$count\\n\"; echo \"literal\\$name\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "name=Ada count=3\nliteral$name\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_lang020_switch_for_interpolation_shape_to_native_binary() {
    let root = temp_dir("ptn-native-lang020-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("lang020-shape.php");
    let output = root.join("lang020-shape-bin");
    fs::write(
        &input,
        "<?php
$i = \"abc\";
for ($j = 0; $j < 10; $j++) {
    switch (1) {
        case 1:
            echo \"In branch 1\\n\";
            switch ($i) {
                case \"ab\":
                    echo \"This doesn't work... :(\\n\";
                    break;
                case \"abcd\":
                    echo \"This works!\\n\";
                    break;
                case \"blah\":
                    echo \"Hmmm, no worki\\n\";
                    break;
                default:
                    echo \"Inner default...\\n\";
            }
            for ($blah = 0; $blah < 200; $blah++) {
                if ($blah == 100) {
                    echo \"blah=$blah\\n\";
                }
            }
            break;
        default:
            echo \"bad\\n\";
            break;
    }
}",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let expected = (0..10)
        .map(|_| "In branch 1\nInner default...\nblah=100\n")
        .collect::<String>();
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), expected);
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
fn compile_boxed_arithmetic_literals_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-literals");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-literals.php");
    let output = root.join("arithmetic-literals-bin");
    fs::write(
        &input,
        "<?php echo 10 - 3, \" \", 2 * 3, \" \", 7 / 2, \" \", 8 % 3, \" \", -5 % 2, \" \", 5 % -2, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "7 6 3.5 2 -1 1\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_numeric_literal_separator_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-numeric-literal-separators");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("numeric-literal-separators.php");
    let output = root.join("numeric-literal-separators-bin");
    fs::write(
        &input,
        "<?php
var_dump(299_792_458 === 299792458);
var_dump(135_00 === 13500);
var_dump(96_485.332_12 === 96485.33212);
var_dump(6.626_070_15e-34 === 6.62607015e-34);
var_dump(6.674_083e-11 === 6.674083e-11);
var_dump(0xCAFE_F00D === 0xCAFEF00D);
var_dump(0x54_4A_42 === 0x544A42);
var_dump(0b0101_1111 === 0b01011111);
var_dump(0b01_0000_10 === 0b01000010);
var_dump(0137_041 === 0137041);
var_dump(0_124 === 0124);
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\n".repeat(11)
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_boxed_arithmetic_variables_and_assignment_results_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-variables");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-variables.php");
    let output = root.join("arithmetic-variables-bin");
    fs::write(
        &input,
        "<?php $left = \"10\"; $right = 3; $difference = $left - $right; $product = $difference * $right; $quotient = $product / 3; $remainder = $product % 5; echo $difference, \" \", $product, \" \", $quotient, \" \", $remainder, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "7 21 7 1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_boxed_arithmetic_chained_precedence_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-precedence");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-precedence.php");
    let output = root.join("arithmetic-precedence-bin");
    fs::write(
        &input,
        "<?php echo 20 - 3 * 4 + 8 / 2 . \" \"; echo (20 - 3) * (4 + 8) / 2 % 7, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "12 4\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_numeric_string_multiplicative_arithmetic_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-conversions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-conversions.php");
    let output = root.join("arithmetic-conversions-bin");
    fs::write(
        &input,
        "<?php echo \"8\" - true, \" \", \"2.5\" * 2, \" \", \"6\" / \"3\", \" \", \"5\" % \"2\", \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "7 5 2 1\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_arithmetic_operands_evaluate_left_to_right_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-left-to-right");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-left-to-right.php");
    let output = root.join("arithmetic-left-to-right-bin");
    fs::write(&input, "<?php echo $left * $right + $third . \"\\n\";").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Warning: Undefined variable $left\nWarning: Undefined variable $right\nWarning: Undefined variable $third\n"
    );
}

#[test]
fn compile_direct_arithmetic_compound_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-compound-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-compound.php");
    let output = root.join("arithmetic-compound-bin");
    fs::write(
        &input,
        "<?php $total = 20; $total -= 4; $total *= 2; $total /= 4; $total %= 5; print $total . \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "3\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_bitwise_scalar_operations_to_native_binary() {
    let root = temp_dir("ptn-native-bitwise-scalars");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-scalars.php");
    let output = root.join("bitwise-scalars-bin");
    fs::write(
        &input,
        "<?php echo 6 & 3, \" \", 4 | 1, \" \", 6 ^ 3, \"\\n\"; var_dump(\"123\" & \"234\"); var_dump(\"323423\" | \"2323.555\"); var_dump(\"some\" | \"test\"); var_dump(bin2hex(\"123\" ^ \"234\")); $s = \"test\"; $s &= \"some long\"; var_dump($s); $o = \"some\"; $o |= \"test\"; var_dump($o); $x = \"some\"; $x ^= \"test long\"; var_dump(bin2hex($x));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2 5 5\nstring(3) \"020\"\nstring(8) \"3337>755\"\nstring(4) \"wo\x7fu\"\nstring(6) \"030107\"\nstring(4) \"pead\"\nstring(4) \"wo\x7fu\"\nstring(8) \"070a1e11\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_unary_bitwise_not_to_native_binary() {
    let root = temp_dir("ptn-native-bitwise-not");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-not.php");
    let output = root.join("bitwise-not-bin");
    fs::write(
        &input,
        "<?php var_dump(~23); var_dump(bin2hex(~\"some\")); var_dump(~23.67);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(-24)\nstring(8) \"8c90929a\"\nint(-24)\n"
    );
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Deprecated: Implicit conversion from float 23.67 to int loses precision in ptn-generated-code on line 0\n"
    );
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
fn compile_unary_plus_precedence_to_native_binary() {
    let root = temp_dir("ptn-native-unary-plus-precedence");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unary-plus-precedence.php");
    let output = root.join("unary-plus-precedence-bin");
    fs::write(&input, "<?php echo 1/-2*5; echo \"\\n\"; echo 6/+2*-3;").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "-2.5\n-9");
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
    let unsupported_operator = parser::parse("<?php $name **= 1;").unwrap_err();
    assert!(unsupported_operator.message.contains("expected assignment"));

    let unsupported_lvalue = parser::parse("<?php $items[0] += 1;").unwrap_err();
    assert!(unsupported_lvalue
        .message
        .contains("unsupported PHP token '['"));
}

#[test]
fn var_dump_complex_edges_remain_unsupported_before_codegen() {
    for source in [
        "<?php var_dump([]);",
        "<?php var_dump(new stdClass);",
        "<?php $array = []; $array[] = &$array; var_dump($array);",
        "<?php $value = 1; $ref =& $value; var_dump($ref);",
    ] {
        assert!(
            parser::parse(source).is_err(),
            "expected unsupported var_dump edge to fail before codegen: {source}"
        );
    }
}

#[test]
fn unsupported_internal_functions_fail_in_generated_runtime() {
    let root = temp_dir("ptn-native-unsupported-internal-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unsupported-internal-function.php");
    let output = root.join("unsupported-internal-function-bin");
    fs::write(&input, "<?php var_dump(fopen('php://memory', 'r'));").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: Call to undefined function fopen()\n"
    );
}

#[test]
fn support_docs_name_var_dump_unsupported_edges() {
    let support = fs::read_to_string("docs/SUPPORT.md").unwrap();
    assert!(support.contains("Arrays, objects, resources, recursive structures, references"));
    assert!(support.contains("Embedded NUL strings"));
    assert!(support.contains("Full PHP float precision and formatting edge cases"));
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}

fn phpc_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_phpc")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/debug/phpc"))
}
