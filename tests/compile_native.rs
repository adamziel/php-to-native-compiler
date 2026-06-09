use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{
    AssignmentOp, BinaryOp, CastKind, Expr, IncDecOp, MagicConstantKind, Statement, StringPart,
    UnaryOp,
};
use ptn::lexer::{self, TokenKind};
use ptn::{compile_file, parser, CompileOptions, DiagnosticKind};

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
        "<?php 299_792_458 96_485.332_12 6.626_070_15e-34 0xCAFE_F00D 0b0101_1111 0137_041 0_124 0o1_6 0O10",
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
    assert!(matches!(tokens[8].kind, TokenKind::Int(14)));
    assert!(matches!(tokens[9].kind, TokenKind::Int(8)));
}

#[test]
fn lexer_rejects_invalid_legacy_octal_integer_literals_as_parse_errors() {
    for source in ["<?php\n$x = 08;", "<?php\n$x = 0_8;", "<?php\n$x = 019;"] {
        let error = lexer::lex(source).unwrap_err();
        assert_eq!(error.message, "Invalid numeric literal");
        assert_eq!(error.kind, DiagnosticKind::ParseError);
        let span = error.span.unwrap();
        assert_eq!(span.line, 2);
        assert_eq!(span.column, 6);
    }
}

#[test]
fn lexer_keeps_leading_zero_floats_decimal() {
    let tokens = lexer::lex("<?php 08.5 08e1 007.25").unwrap();
    assert!(matches!(tokens[1].kind, TokenKind::Float(value) if value == 8.5));
    assert!(matches!(tokens[2].kind, TokenKind::Float(value) if value == 80.0));
    assert!(matches!(tokens[3].kind, TokenKind::Float(value) if value == 7.25));
}

#[test]
fn lexer_preserves_unknown_string_escape_backslashes() {
    let tokens = lexer::lex("<?php \"\\+\" '\\t' \"\\$\" \"\\n\"").unwrap();
    assert!(matches!(&tokens[1].kind, TokenKind::String(value) if value == "\\+"));
    assert!(matches!(&tokens[2].kind, TokenKind::String(value) if value == "\\t"));
    assert!(matches!(&tokens[3].kind, TokenKind::String(value) if value == "$"));
    assert!(matches!(&tokens[4].kind, TokenKind::String(value) if value == "\n"));
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
fn parser_accepts_power_as_right_associative_above_unary() {
    let program = parser::parse("<?php echo -3 ** 2, 2 ** 3 ** 2;").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    let Expr::Unary {
        op: UnaryOp::Negate,
        expr,
        ..
    } = &expressions[0]
    else {
        panic!("expected unary negation around power expression");
    };
    assert!(matches!(
        expr.as_ref(),
        Expr::Binary {
            op: BinaryOp::Power,
            ..
        }
    ));

    let Expr::Binary {
        op: BinaryOp::Power,
        right,
        ..
    } = &expressions[1]
    else {
        panic!("expected outer power expression");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Power,
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
        "<?php $value = 1; $value += 2; $value -= 3; $value *= 4; $value **= 2; $value /= 5; $value %= 6; $value .= \"7\"; $value &= \"8\"; $value |= \"9\"; $value ^= \"10\"; $value <<= 11; $value >>= 12;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 13);

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
        panic!("expected power assignment statement");
    };
    assert_eq!(*op, AssignmentOp::PowerAssign);

    let Statement::Assign { op, .. } = &program.statements[5] else {
        panic!("expected divide assignment statement");
    };
    assert_eq!(*op, AssignmentOp::DivideAssign);

    let Statement::Assign { op, .. } = &program.statements[6] else {
        panic!("expected modulo assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ModuloAssign);

    let Statement::Assign { op, .. } = &program.statements[7] else {
        panic!("expected concat assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ConcatAssign);

    let Statement::Assign { op, .. } = &program.statements[8] else {
        panic!("expected bitwise and assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseAndAssign);

    let Statement::Assign { op, .. } = &program.statements[9] else {
        panic!("expected bitwise or assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseOrAssign);

    let Statement::Assign { op, .. } = &program.statements[10] else {
        panic!("expected bitwise xor assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseXorAssign);

    let Statement::Assign { op, .. } = &program.statements[11] else {
        panic!("expected shift left assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ShiftLeftAssign);

    let Statement::Assign { op, .. } = &program.statements[12] else {
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
    assert_eq!(decrement.message, "syntax error, unexpected token \"--\"");
    assert_eq!(decrement.kind, DiagnosticKind::ParseError);

    let invalid_prefix = parser::parse("<?php ++1;").unwrap_err();
    assert!(invalid_prefix.message.contains("expected variable"));
}

#[test]
fn parser_reports_unexpected_tokens_with_parse_error_spans() {
    let brace = parser::parse("<?php\nvar_dump($foo{0});").unwrap_err();
    assert_eq!(
        brace.message,
        "syntax error, unexpected token \"{\", expecting \")\""
    );
    assert_eq!(brace.kind, DiagnosticKind::ParseError);
    let span = brace.span.unwrap();
    assert_eq!(span.line, 2);
    assert_eq!(span.column, 14);

    let integer = parser::parse("<?php\n$foo = (mixed) 12;").unwrap_err();
    assert_eq!(integer.message, "syntax error, unexpected integer \"12\"");
    assert_eq!(integer.kind, DiagnosticKind::ParseError);
    let span = integer.span.unwrap();
    assert_eq!(span.line, 2);
    assert_eq!(span.column, 16);
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
fn parser_distinguishes_non_canonical_boolean_cast() {
    let program = parser::parse("<?php var_dump((boolean) 42, (bool) 42);").unwrap();
    let Statement::Call { arguments, .. } = &program.statements[0] else {
        panic!("expected call statement");
    };
    assert!(matches!(
        &arguments[0],
        Expr::Cast {
            kind: CastKind::Boolean,
            ..
        }
    ));
    assert!(matches!(
        &arguments[1],
        Expr::Cast {
            kind: CastKind::Bool,
            ..
        }
    ));
}

#[test]
fn parser_distinguishes_non_canonical_scalar_casts() {
    let program = parser::parse(
        "<?php var_dump((integer) 42, (int) 42, (double) 42, (float) 42, (binary) 42, (string) 42);",
    )
    .unwrap();
    let Statement::Call { arguments, .. } = &program.statements[0] else {
        panic!("expected call statement");
    };
    let expected = [
        CastKind::Integer,
        CastKind::Int,
        CastKind::Double,
        CastKind::Float,
        CastKind::Binary,
        CastKind::String,
    ];

    for (argument, expected_kind) in arguments.iter().zip(expected) {
        let Expr::Cast { kind, .. } = argument else {
            panic!("expected cast argument");
        };
        assert_eq!(*kind, expected_kind);
    }
}

#[test]
fn parser_rejects_removed_real_cast_with_parse_error_kind() {
    let error = parser::parse("<?php var_dump((real) 42);").unwrap_err();
    assert_eq!(
        error.message,
        "The (real) cast has been removed, use (float) instead"
    );
    assert_eq!(error.kind, DiagnosticKind::ParseError);
    assert_eq!(error.span.unwrap().line, 1);
}

#[test]
fn parser_rejects_removed_unset_cast_with_php_message() {
    let error = parser::parse("<?php var_dump((unset) $x);").unwrap_err();
    assert_eq!(error.message, "The (unset) cast is no longer supported");
    assert_eq!(error.kind, DiagnosticKind::Fatal);
    assert_eq!(error.span.unwrap().line, 1);
}

#[test]
fn parser_rejects_void_cast_expression_context_with_parse_error_kind() {
    let error = parser::parse("<?php $tmp = (void)$dummy;").unwrap_err();
    assert_eq!(error.message, "syntax error, unexpected token \"(void)\"");
    assert_eq!(error.kind, DiagnosticKind::ParseError);
    assert_eq!(error.span.unwrap().line, 1);
}

#[test]
fn parser_rejects_unterminated_block_comment_with_parse_error_kind() {
    let error = parser::parse("<?php\n/* Foo\nBar").unwrap_err();
    assert_eq!(error.message, "Unterminated comment starting line 2");
    assert_eq!(error.kind, DiagnosticKind::ParseError);
    assert_eq!(error.span.unwrap().line, 2);
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
fn parser_accepts_array_literals_and_spaceship_expressions() {
    let program = parser::parse("<?php var_dump([1, \"2\" => 3, 4 => [5]] <=> []);").unwrap();
    let Statement::Call {
        name, arguments, ..
    } = &program.statements[0]
    else {
        panic!("expected var_dump statement");
    };
    assert_eq!(name, "var_dump");

    let Expr::Binary {
        op: BinaryOp::Spaceship,
        left,
        right,
        ..
    } = &arguments[0]
    else {
        panic!("expected spaceship expression");
    };
    let Expr::Array { elements, .. } = left.as_ref() else {
        panic!("expected left array literal");
    };
    assert_eq!(elements.len(), 3);
    assert!(elements[0].key.is_none());
    assert!(elements[1].key.is_some());
    assert!(matches!(&elements[2].value, Expr::Array { .. }));
    assert!(matches!(right.as_ref(), Expr::Array { elements, .. } if elements.is_empty()));
}

#[test]
fn parser_accepts_array_read_expressions() {
    let program = parser::parse(
        "<?php echo $items[\"7\"], ([1, 2])[0], $matrix[0][\"name\"], [\"x\" => 4][\"x\"];",
    )
    .unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert_eq!(expressions.len(), 4);
    assert!(matches!(
        &expressions[0],
        Expr::ArrayAccess {
            array,
            index,
            ..
        } if matches!(array.as_ref(), Expr::Variable(name, _) if name == "items")
            && matches!(index.as_ref(), Expr::String(value, _) if value == "7")
    ));
    assert!(matches!(
        &expressions[1],
        Expr::ArrayAccess { array, .. }
            if matches!(array.as_ref(), Expr::Grouped { .. })
    ));
    assert!(matches!(
        &expressions[2],
        Expr::ArrayAccess { array, .. }
            if matches!(array.as_ref(), Expr::ArrayAccess { .. })
    ));
    assert!(matches!(
        &expressions[3],
        Expr::ArrayAccess { array, .. }
            if matches!(array.as_ref(), Expr::Array { .. })
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
fn parser_accepts_global_const_declarations() {
    let program = parser::parse(
        "<?php const C = 0 && __NAMESPACE__, D = PHP_EOL, A = [\"x\" => 1]; var_dump(C, defined(\"D\"));",
    )
    .unwrap();

    let Statement::Const { declarations, .. } = &program.statements[0] else {
        panic!("expected const declaration");
    };
    assert_eq!(declarations.len(), 3);
    assert_eq!(declarations[0].name, "C");
    assert!(matches!(
        &declarations[0].value,
        Expr::Binary {
            op: BinaryOp::And,
            ..
        }
    ));
    assert!(matches!(
        &declarations[1].value,
        Expr::Constant(name, _) if name == "PHP_EOL"
    ));
    assert!(matches!(
        &declarations[2].value,
        Expr::Array { elements, .. } if elements.len() == 1
    ));
}

#[test]
fn parser_rejects_dynamic_global_const_initializer() {
    let error = parser::parse("<?php const C = $value;").unwrap_err();
    assert!(error
        .to_string()
        .contains("constant expression contains invalid operation"));
}

#[test]
fn parser_rejects_nested_global_const_declaration() {
    let error = parser::parse("<?php if (true) const C = 1;").unwrap_err();
    assert!(error
        .to_string()
        .contains("constant declarations must be at global scope"));
}

#[test]
fn parser_accepts_global_magic_constants() {
    let program = parser::parse(
        "<?php var_dump(__LINE__, __FILE__, __DIR__, __FUNCTION__, __METHOD__, __CLASS__, __TRAIT__, __NAMESPACE__);",
    )
    .unwrap();
    let Statement::Call { arguments, .. } = &program.statements[0] else {
        panic!("expected call statement");
    };
    assert!(matches!(
        &arguments[0],
        Expr::MagicConstant(MagicConstantKind::Line, _)
    ));
    assert!(matches!(
        &arguments[1],
        Expr::MagicConstant(MagicConstantKind::File, _)
    ));
    assert!(matches!(
        &arguments[2],
        Expr::MagicConstant(MagicConstantKind::Dir, _)
    ));
    assert!(matches!(
        &arguments[3],
        Expr::MagicConstant(MagicConstantKind::Function, _)
    ));
    assert!(matches!(
        &arguments[4],
        Expr::MagicConstant(MagicConstantKind::Method, _)
    ));
    assert!(matches!(
        &arguments[5],
        Expr::MagicConstant(MagicConstantKind::Class, _)
    ));
    assert!(matches!(
        &arguments[6],
        Expr::MagicConstant(MagicConstantKind::Trait, _)
    ));
    assert!(matches!(
        &arguments[7],
        Expr::MagicConstant(MagicConstantKind::Namespace, _)
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
fn parser_accepts_labels_goto_and_single_statement_if() {
    let program =
        parser::parse("<?php $n = 1; L1: if ($n <= 3) goto L1; else echo \"done\\n\";").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(
        &program.statements[1],
        Statement::Label { name, .. } if name == "L1"
    ));
    let Statement::If {
        then_body,
        else_body,
        ..
    } = &program.statements[2]
    else {
        panic!("expected if statement");
    };
    assert!(matches!(
        &then_body[0],
        Statement::Goto { label, .. } if label == "L1"
    ));
    assert!(matches!(&else_body[0], Statement::Echo { .. }));
}

#[test]
fn parser_accepts_plain_statement_blocks_and_return() {
    let program = parser::parse(
        "<?php
goto A;
{
    B:
        goto C;
        return;
}
A:
    goto B;
{
    C:
    {
        print \"Done!\\n\";
    }
}
",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 5);
    assert!(matches!(&program.statements[1], Statement::Block { .. }));
    let Statement::Block { statements, .. } = &program.statements[1] else {
        panic!("expected block statement");
    };
    assert!(matches!(&statements[0], Statement::Label { name, .. } if name == "B"));
    assert!(matches!(&statements[1], Statement::Goto { label, .. } if label == "C"));
    assert!(matches!(
        &statements[2],
        Statement::Return { value: None, .. }
    ));
    let Statement::Block { statements, .. } = &program.statements[4] else {
        panic!("expected outer block statement");
    };
    assert!(matches!(&statements[0], Statement::Label { name, .. } if name == "C"));
    assert!(matches!(&statements[1], Statement::Block { .. }));
}

#[test]
fn parser_rejects_goto_to_undefined_label() {
    let error = parser::parse("<?php\ngoto L1;\n").unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Fatal);
    assert_eq!(error.message, "'goto' to undefined label 'L1'");
    assert_eq!(error.span.unwrap().line, 2);
}

#[test]
fn parser_rejects_duplicate_labels() {
    let error = parser::parse("<?php\nfoo:\necho 1;\nfoo:\necho 2;\n").unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Fatal);
    assert_eq!(error.message, "Label 'foo' already defined");
    assert_eq!(error.span.unwrap().line, 4);
}

#[test]
fn parser_rejects_goto_into_loop_or_switch() {
    for (source, line) in [
        ("<?php\nwhile (0) {\n    L1: echo \"bug\\n\";\n}\ngoto L1;\n", 5),
        ("<?php\ngoto L1;\nwhile (0) {\n    L1: echo \"bug\\n\";\n}\n", 2),
        (
            "<?php\nswitch (0) {\n    case 1:\n        L1: echo \"bug\\n\";\n        break;\n}\ngoto L1;\n",
            7,
        ),
        (
            "<?php\ngoto L1;\nswitch (0) {\n    case 1:\n        L1: echo \"bug\\n\";\n        break;\n}\n",
            2,
        ),
    ] {
        let error = parser::parse(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Fatal);
        assert_eq!(
            error.message,
            "'goto' into loop or switch statement is disallowed"
        );
        assert_eq!(error.span.unwrap().line, line);
    }
}

#[test]
fn parser_accepts_goto_leaving_loop_and_within_same_loop() {
    parser::parse(
        "<?php
$s = \"X\";
L1: if ($s != \"X\") {
    echo \"done\\n\";
} else {
    while ($s != \"XXX\") {
        $s .= \"X\";
        goto L1;
    }
}
",
    )
    .unwrap();

    parser::parse(
        "<?php
do {
    if (1) {
        goto L1;
    } else {
L1:
        echo \"ok\\n\";
    }
} while (0);
",
    )
    .unwrap();
}

#[test]
fn parser_accepts_braced_switch_cases_default_and_break() {
    let program = parser::parse(
        "<?php $a = 1; switch ($a) { case 0: echo \"bad\"; break; case 1: echo \"good\"; break 2; default: echo \"bad\"; break; }",
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
        Some(Statement::Break { level: 2, .. })
    ));
}

#[test]
fn parser_accepts_single_statement_loop_bodies_and_break_levels() {
    let program = parser::parse(
        "<?php for (;;) break 2147483648; while (false) echo \"bad\"; do print \"once\"; while (false);",
    )
    .unwrap();

    assert_eq!(program.statements.len(), 3);
    let Statement::For { body, .. } = &program.statements[0] else {
        panic!("expected for statement");
    };
    assert!(matches!(
        &body[0],
        Statement::Break {
            level: 2_147_483_648usize,
            ..
        }
    ));
    let Statement::While { body, .. } = &program.statements[1] else {
        panic!("expected while statement");
    };
    assert!(matches!(&body[0], Statement::Echo { .. }));
    let Statement::DoWhile { body, .. } = &program.statements[2] else {
        panic!("expected do while statement");
    };
    assert!(matches!(&body[0], Statement::Print { .. }));
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
fn phpc_renders_undefined_goto_label_as_php_fatal() {
    let root = temp_dir("ptn-phpc-undefined-goto-label");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("jump06.php");
    fs::write(&input, "<?php\ngoto L1;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: 'goto' to undefined label 'L1' in {} on line 2\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_duplicate_label_as_php_fatal() {
    let root = temp_dir("ptn-phpc-duplicate-label");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("duplicate-label.php");
    fs::write(&input, "<?php\nfoo:\necho 1;\nfoo:\necho 2;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Label 'foo' already defined in {} on line 4\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_goto_into_loop_or_switch_as_php_fatal() {
    let cases = [
        (
            "jump08.php",
            "<?php\ngoto L1;\nwhile (0) {\n    L1: echo \"bug\\n\";\n}\n",
            2,
        ),
        (
            "jump10.php",
            "<?php\ngoto L1;\nswitch (0) {\n    case 1:\n        L1: echo \"bug\\n\";\n        break;\n}\n",
            2,
        ),
    ];

    for (name, source, line) in cases {
        let root_name = format!("ptn-phpc-goto-restriction-{name}");
        let root = temp_dir(&root_name);
        fs::create_dir_all(&root).unwrap();
        let input = root.join(name);
        fs::write(&input, source).unwrap();

        let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
        assert!(!execution.status.success());
        assert_eq!(execution.status.code(), Some(255));
        assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
        assert_eq!(
            String::from_utf8(execution.stderr).unwrap(),
            format!(
                "Fatal error: 'goto' into loop or switch statement is disallowed in {} on line {line}\n",
                input.display()
            )
        );
    }
}

#[test]
fn phpc_renders_spanned_parse_diagnostics_as_php_parse_errors() {
    let root = temp_dir("ptn-phpc-parse-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("real-cast.php");
    fs::write(&input, "<?php\n\nvar_dump((real) 42);\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: The (real) cast has been removed, use (float) instead in {} on line 3\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_removed_unset_cast_as_php_fatal() {
    let root = temp_dir("ptn-phpc-unset-cast-fatal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unset-cast.php");
    fs::write(&input, "<?php\n\n$x = 1;\nvar_dump((unset) $x);\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: The (unset) cast is no longer supported in {} on line 4\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_void_cast_expression_context_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-void-cast-expression-parse-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("void-cast-expression.php");
    fs::write(&input, "<?php\n\n$tmp = (void)$dummy;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: syntax error, unexpected token \"(void)\" in {} on line 3\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_unterminated_block_comment_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-unterminated-block-comment-parse-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unterminated-comment.php");
    fs::write(&input, "<?php\n/* Foo\nBar").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: Unterminated comment starting line 2 in {} on line 2\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_invalid_legacy_octal_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-invalid-legacy-octal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("invalid-octal.php");
    fs::write(&input, "<?php\n\n$x = 08;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: Invalid numeric literal in {} on line 3\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_unexpected_right_paren_token_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-unexpected-right-paren-token");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unexpected-right-paren-token.php");
    fs::write(&input, "<?php\n$foo = 'BAR';\nvar_dump($foo{0});\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: syntax error, unexpected token \"{{\", expecting \")\" in {} on line 3\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_unexpected_statement_token_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-unexpected-statement-token");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unexpected-statement-token.php");
    fs::write(&input, "<?php\n\n$foo = (mixed) 12;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: syntax error, unexpected integer \"12\" in {} on line 3\n",
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
fn compile_str_contains_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-str-contains-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-contains.php");
    let output = root.join("str-contains-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(str_contains(\"test string\", \"test\"));\n\
var_dump(str_contains(\"test string\", \"string\"));\n\
var_dump(str_contains(\"test string\", \"strin\"));\n\
var_dump(str_contains(\"test string\", \"t s\"));\n\
var_dump(str_contains(\"test string\", \"g\"));\n\
var_dump(str_contains(\"te\".chr(0).\"st\", chr(0)));\n\
var_dump(str_contains(\"tEst\", \"test\"));\n\
var_dump(str_contains(\"teSt\", \"test\"));\n\
var_dump(str_contains(\"\", \"\"));\n\
var_dump(str_contains(\"a\", \"\"));\n\
var_dump(str_contains(\"\", \"a\"));\n\
var_dump(str_contains(\"\\\\\\\\a\", \"\\\\a\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_str_contains_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-str-contains-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-contains-registry.php");
    let output = root.join("str-contains-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(str_contains(12345, \"34\"), str_contains(false, \"\"), str_contains(\"abc\", \"D\"), function_exists(\"str_contains\"), function_exists(\"STR_CONTAINS\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_fdiv_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-fdiv-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("fdiv.php");
    let output = root.join("fdiv-bin");
    fs::write(
        &input,
        "<?php\n\
\n\
var_dump(fdiv(10, 3));\n\
var_dump(fdiv(10., 3.));\n\
var_dump(fdiv(-10., 2.5));\n\
var_dump(fdiv(10., -2.5));\n\
echo \"\\n\";\n\
var_dump(fdiv(10., 0.));\n\
var_dump(fdiv(10., -0.));\n\
var_dump(fdiv(-10., 0.));\n\
var_dump(fdiv(-10., -0.));\n\
echo \"\\n\";\n\
var_dump(fdiv(INF, 0.));\n\
var_dump(fdiv(INF, -0.));\n\
var_dump(fdiv(-INF, 0.));\n\
var_dump(fdiv(-INF, -0.));\n\
echo \"\\n\";\n\
var_dump(fdiv(0., 0.));\n\
var_dump(fdiv(0., -0.));\n\
var_dump(fdiv(-0., 0.));\n\
var_dump(fdiv(-0., -0.));\n\
echo \"\\n\";\n\
var_dump(fdiv(INF, INF));\n\
var_dump(fdiv(INF, -INF));\n\
var_dump(fdiv(-INF, INF));\n\
var_dump(fdiv(-INF, -INF));\n\
echo \"\\n\";\n\
var_dump(fdiv(0., INF));\n\
var_dump(fdiv(0., -INF));\n\
var_dump(fdiv(-0., INF));\n\
var_dump(fdiv(-0., -INF));\n\
echo \"\\n\";\n\
var_dump(fdiv(NAN, NAN));\n\
var_dump(fdiv(INF, NAN));\n\
var_dump(fdiv(0., NAN));\n\
var_dump(fdiv(NAN, INF));\n\
var_dump(fdiv(NAN, 0.));\n\
\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(3.3333333333333335)\nfloat(3.3333333333333335)\nfloat(-4)\nfloat(-4)\n\nfloat(INF)\nfloat(-INF)\nfloat(-INF)\nfloat(INF)\n\nfloat(INF)\nfloat(-INF)\nfloat(-INF)\nfloat(INF)\n\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\n\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\n\nfloat(0)\nfloat(-0)\nfloat(-0)\nfloat(0)\n\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\nfloat(NAN)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_fdiv_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-fdiv-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("fdiv-registry.php");
    let output = root.join("fdiv-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(fdiv(\"9\", \"2\"), fdiv(true, 2), function_exists(\"fdiv\"), function_exists(\"FDIV\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(4.5)\nfloat(0.5)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_quotemeta_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-quotemeta-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("quotemeta-basic.php");
    let output = root.join("quotemeta-basic-bin");
    fs::write(
        &input,
        r#"<?php

echo "*** Testing quotemeta() : basic functionality ***\n";

var_dump(quotemeta("Hello how are you ?"));
var_dump(quotemeta("(100 + 50) * 10"));
var_dump(quotemeta("\+*?[^]($)"));
?>"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "*** Testing quotemeta() : basic functionality ***\n",
            r#"string(20) "Hello how are you \?""#,
            "\n",
            r#"string(19) "\(100 \+ 50\) \* 10""#,
            "\n",
            r#"string(20) "\\\+\*\?\[\^\]\(\$\)""#,
            "\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_quotemeta_empty_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-quotemeta-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("quotemeta-registry.php");
    let output = root.join("quotemeta-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(quotemeta(\"\"), quotemeta(123.5), quotemeta(true), quotemeta(false), function_exists(\"quotemeta\"), function_exists(\"QUOTEMETA\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(0) \"\"\nstring(6) \"123\\.5\"\nstring(1) \"1\"\nstring(0) \"\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_md5_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-md5-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("md5.php");
    let output = root.join("md5-bin");
    fs::write(
        &input,
        "<?php\n\
echo md5(\"\").\"\\n\";\n\
echo md5(\"a\").\"\\n\";\n\
echo md5(\"abc\").\"\\n\";\n\
echo md5(\"message digest\").\"\\n\";\n\
echo md5(\"abcdefghijklmnopqrstuvwxyz\").\"\\n\";\n\
echo md5(\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\").\"\\n\";\n\
echo md5(\"12345678901234567890123456789012345678901234567890123456789012345678901234567890\").\"\\n\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "d41d8cd98f00b204e9800998ecf8427e\n0cc175b9c0f1b6a831c399e269772661\n900150983cd24fb0d6963f7d28e17f72\nf96b697d7cb7938d525a2f31aaf161d0\nc3fcd3d76192e4007dfb496cca67e13b\nd174ab98d277d9f5a5611c2c9f419d9f\n57edf4a22be3c955ac49da2e2107b67a\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_md5_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-md5-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("md5-registry.php");
    let output = root.join("md5-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(md5(123), md5(true), function_exists(\"md5\"), function_exists(\"MD5\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(32) \"202cb962ac59075b964b07152d234b70\"\nstring(32) \"c4ca4238a0b923820dcc509a6f75849b\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_md5_raw_output_argument_to_native_binary() {
    let root = temp_dir("ptn-native-md5-raw-output-argument");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("md5-raw-output.php");
    let output = root.join("md5-raw-output-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing md5() : basic functionality - with raw output***\\n\";\n\
$str = \"Hello World\";\n\
$md5_raw = md5($str, true);\n\
var_dump(bin2hex($md5_raw));\n\
\n\
$md5 = md5($str, false);\n\
\n\
if (strcmp(bin2hex($md5_raw), $md5) == 0 ) {\n\
    echo \"TEST PASSED\\n\";\n\
} else {\n\
    echo \"TEST FAILED\\n\";\n\
    var_dump($md5_raw, $md5);\n\
}\n\
\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing md5() : basic functionality - with raw output***\nstring(32) \"b10a8db164e0754105b7a99be72e3fe5\"\nTEST PASSED\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_sha1_basic_and_raw_output_to_native_binary() {
    let root = temp_dir("ptn-native-sha1-basic-and-raw-output");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sha1-basic.php");
    let output = root.join("sha1-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing sha1() : basic functionality ***\\n\";\n\
\n\
echo \"\\n-- Without raw argument --\\n\";\n\
var_dump(sha1(\"\"));\n\
var_dump(sha1(\"a\"));\n\
var_dump(sha1(\"abc\"));\n\
var_dump(sha1(\"message digest\"));\n\
var_dump(sha1(\"abcdefghijklmnopqrstuvwxyz\"));\n\
var_dump(sha1(\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\"));\n\
var_dump(sha1(\"12345678901234567890123456789012345678901234567890123456789012345678901234567890\"));\n\
\n\
echo \"\\n-- With raw == false --\\n\";\n\
var_dump(sha1(\"\", false));\n\
var_dump(sha1(\"a\", false));\n\
var_dump(sha1(\"abc\", false));\n\
var_dump(sha1(\"message digest\", false));\n\
var_dump(sha1(\"abcdefghijklmnopqrstuvwxyz\", false));\n\
var_dump(sha1(\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\", false));\n\
var_dump(sha1(\"12345678901234567890123456789012345678901234567890123456789012345678901234567890\", false));\n\
\n\
echo \"\\n-- With raw == true --\\n\";\n\
var_dump(bin2hex(sha1(\"\", true)));\n\
var_dump(bin2hex(sha1(\"a\", true)));\n\
var_dump(bin2hex(sha1(\"abc\", true)));\n\
var_dump(bin2hex(sha1(\"message digest\", true)));\n\
var_dump(bin2hex(sha1(\"abcdefghijklmnopqrstuvwxyz\", true)));\n\
var_dump(bin2hex(sha1(\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\", true)));\n\
var_dump(bin2hex(sha1(\"12345678901234567890123456789012345678901234567890123456789012345678901234567890\", true)));\n\
var_dump(function_exists(\"sha1\"), function_exists(\"SHA1\"));\n\
\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing sha1() : basic functionality ***\n\n-- Without raw argument --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\n\n-- With raw == false --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\n\n-- With raw == true --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_substr_int_min_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-substr-int-min-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("substr-int-min.php");
    let output = root.join("substr-int-min-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(substr('x', PHP_INT_MIN));\n\
var_dump(substr('x', 0, PHP_INT_MIN));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"x\"\nstring(0) \"\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_substr_scalar_conversion_and_bounds_to_native_binary() {
    let root = temp_dir("ptn-native-substr-scalar-conversion-and-bounds");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("substr-scalar-conversion.php");
    let output = root.join("substr-scalar-conversion-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(substr(\"abcdef\", 1));\n\
var_dump(substr(\"abcdef\", -2));\n\
var_dump(substr(\"abcdef\", 1, 3));\n\
var_dump(substr(\"abcdef\", 1, -3));\n\
var_dump(substr(\"abcdef\", 4, -4));\n\
var_dump(substr(\"abcdef\", -8));\n\
var_dump(substr(\"abcdef\", 2, null));\n\
var_dump(substr(12345, \"1\", \"3\"));\n\
var_dump(function_exists(\"substr\"), function_exists(\"SUBSTR\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"bcdef\"\nstring(2) \"ef\"\nstring(3) \"bcd\"\nstring(2) \"bc\"\nstring(0) \"\"\nstring(6) \"abcdef\"\nstring(4) \"cdef\"\nstring(3) \"234\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_hex2bin_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-hex2bin-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("hex2bin-basic.php");
    let output = root.join("hex2bin-basic-bin");
    fs::write(
        &input,
        "<?php\n\
\n\
var_dump(bin2hex(hex2bin('012345')) == '012345');\n\
var_dump(bin2hex(hex2bin('abc123')) == 'abc123');\n\
var_dump(bin2hex(hex2bin('123abc')) == '123abc');\n\
var_dump(bin2hex(hex2bin('FFFFFF')) == 'ffffff');\n\
var_dump(function_exists(\"hex2bin\"), function_exists(\"HEX2BIN\"));\n\
\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_hex2bin_invalid_input_to_native_binary() {
    let root = temp_dir("ptn-native-hex2bin-invalid-input");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("hex2bin-invalid.php");
    let output = root.join("hex2bin-invalid-bin");
    fs::write(
        &input,
        "<?php var_dump(hex2bin('f')); var_dump(hex2bin('zz'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: hex2bin(): Hexadecimal input string must have an even length in ptn on line 1\nbool(false)\nWarning: hex2bin(): Input string must be hexadecimal string in ptn on line 1\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_dir_constant_normal_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-dir-constant-normal-phpt-shape");
    let source_dir = root.join("tests").join("constants");
    fs::create_dir_all(&source_dir).unwrap();
    let input = source_dir.join("dir-constant-normal.php");
    let output = root.join("dir-constant-normal-bin");
    fs::write(
        &input,
        "<?php\n\
echo __DIR__ . \"\\n\";\n\
echo dirname(__FILE__) . \"\\n\";\n\
var_dump(function_exists(\"dirname\"), function_exists(\"DIRNAME\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let expected_dir = source_dir.to_string_lossy();
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!("{expected_dir}\n{expected_dir}\nbool(true)\nbool(true)\n")
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_soundex_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-soundex-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("soundex-basic.php");
    let output = root.join("soundex-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing soundex() : basic functionality ***\\n\";\n\
\n\
var_dump(soundex(\"Euler\"));\n\
var_dump(soundex(\"Gauss\"));\n\
var_dump(soundex(\"Hilbert\"));\n\
var_dump(soundex(\"Knuth\"));\n\
var_dump(soundex(\"Lloyd\"));\n\
var_dump(soundex(\"Lukasiewicz\"));\n\
\n\
var_dump(soundex(\"Euler\")       == soundex(\"Ellery\"));\n\
var_dump(soundex(\"Gauss\")       == soundex(\"Ghosh\"));\n\
var_dump(soundex(\"Hilbert\")     == soundex(\"Heilbronn\"));\n\
var_dump(soundex(\"Knuth\")       == soundex(\"Kant\"));\n\
var_dump(soundex(\"Lloyd\")       == soundex(\"Ladd\"));\n\
var_dump(soundex(\"Lukasiewicz\") == soundex(\"Lissajous\"));\n\
\n\
var_dump(soundex(\"Lukasiewicz\") == soundex(\"Ghosh\"));\n\
var_dump(soundex(\"Hilbert\") == soundex(\"Ladd\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing soundex() : basic functionality ***\nstring(4) \"E460\"\nstring(4) \"G200\"\nstring(4) \"H416\"\nstring(4) \"K530\"\nstring(4) \"L300\"\nstring(4) \"L222\"\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_soundex_registry_and_reset_edges_to_native_binary() {
    let root = temp_dir("ptn-native-soundex-registry-and-reset-edges");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("soundex-registry.php");
    let output = root.join("soundex-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(soundex(\"Ashcraft\"), soundex(\"Tymczak\"), soundex(\"Pfister\"), soundex(\"123\"), function_exists(\"soundex\"), function_exists(\"SOUNDEX\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"A226\"\nstring(4) \"T522\"\nstring(4) \"P236\"\nstring(4) \"0000\"\nbool(true)\nbool(true)\n"
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
fn compile_global_magic_constants_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-global-magic-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("magic-constants.php");
    let output = root.join("magic-constants-bin");
    fs::write(
        &input,
        "<?php\n\
\n\
var_dump(\n\
    __LINE__,\n\
    __FILE__,\n\
    __DIR__,\n\
    __FUNCTION__,\n\
    __METHOD__,\n\
    __CLASS__,\n\
    __TRAIT__,\n\
    __NAMESPACE__\n\
);\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let file = input.to_string_lossy();
    let dir = root.to_string_lossy();
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "int(4)\nstring({}) \"{}\"\nstring({}) \"{}\"\nstring(0) \"\"\nstring(0) \"\"\nstring(0) \"\"\nstring(0) \"\"\nstring(0) \"\"\n",
            file.len(),
            file,
            dir.len(),
            dir
        )
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
fn compile_const_eval_and_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-const-eval-and");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("const-eval-and.php");
    let output = root.join("const-eval-and-bin");
    fs::write(
        &input,
        "<?php\n\
const C = 0 && __namespace__;\n\
var_dump(C);\n\
var_dump(defined(\"C\"), defined(\"c\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(false)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_runtime_define_and_constant_to_native_binary() {
    let root = temp_dir("ptn-native-runtime-define-constant");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("runtime-define-constant.php");
    let output = root.join("runtime-define-constant-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(function_exists(\"define\"), function_exists(\"CONSTANT\"));\n\
define(\"USER_CONST\", \"value\");\n\
define(1, 2);\n\
define(\"\", 3);\n\
var_dump(defined(\"USER_CONST\"), constant(\"USER_CONST\"), constant(1), constant(\"\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nstring(5) \"value\"\nint(2)\nint(3)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_duplicate_define_warns_and_preserves_original_constant() {
    let root = temp_dir("ptn-native-duplicate-define");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("duplicate-define.php");
    let output = root.join("duplicate-define-bin");
    fs::write(
        &input,
        "<?php\n\
define(\"dup\", 1);\n\
var_dump(define(\"dup\", 2));\n\
var_dump(constant(\"dup\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: Constant dup already defined, this will be an error in PHP 9 in ptn on line 3\nbool(false)\nint(1)\n"
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
fn compile_array_spaceship_and_identity_to_native_binary() {
    let root = temp_dir("ptn-native-array-spaceship-identity");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-spaceship-identity.php");
    let output = root.join("array-spaceship-identity-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump([1, 2, 3] <=> []);\n\
var_dump([] <=> [1, 2, 3]);\n\
var_dump([1] <=> [2, 3]);\n\
var_dump([1, 2] <=> [1, 3]);\n\
var_dump([1, 3] <=> [1, 2]);\n\
var_dump([1] == [\"1\"]);\n\
var_dump([1] === [\"1\"]);\n\
var_dump([0 => 0] === [\"\" => 0]);\n\
var_dump([0 => 0] === [0x100000000 => 0]);\n\
var_dump([1 => \"a\", 0 => \"b\"] == [0 => \"b\", 1 => \"a\"]);\n\
var_dump([1 => \"a\", 0 => \"b\"] === [0 => \"b\", 1 => \"a\"]);\n\
var_dump([\"0\" => 7, \"\" => 8, 0 => 9]);\n\
var_dump(gettype([]));\n\
var_dump(is_scalar([]));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(-1)\nint(-1)\nint(-1)\nint(1)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\narray(2) {\n  [0]=>\n  int(9)\n  [\"\"]=>\n  int(8)\n}\nstring(5) \"array\"\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_read_expressions_to_native_binary() {
    let root = temp_dir("ptn-native-array-read-expressions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-read-expressions.php");
    let output = root.join("array-read-expressions-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [10, 20, \"7\" => \"seven\", \"\" => \"empty\", true => \"bool-key\"];\n\
$key = \"7\";\n\
var_dump($items[0]);\n\
var_dump($items[\"0\"]);\n\
var_dump([10, 20][1.0]);\n\
var_dump($items[$key]);\n\
var_dump($items[null]);\n\
var_dump($items[true]);\n\
var_dump(([\"nested\" => [2 => \"ok\"]])[\"nested\"][2]);\n\
echo [1, 2, 3][1], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(10)\nint(10)\nint(20)\nstring(5) \"seven\"\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 8\nstring(5) \"empty\"\nstring(8) \"bool-key\"\nstring(2) \"ok\"\n2\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_read_diagnostics_to_native_binary() {
    let root = temp_dir("ptn-native-array-read-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-read-diagnostics.php");
    let output = root.join("array-read-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [];\n\
var_dump($items[\"7.5\"]);\n\
var_dump($items[0]);\n\
$value = 1;\n\
var_dump($value[0]);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nWarning: Undefined array key \"7.5\" in ptn on line 3\nNULL\n\nWarning: Undefined array key 0 in ptn on line 4\nNULL\n\nWarning: Trying to access array offset on value of type int in ptn on line 6\nNULL\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_offset_reads_to_native_binary() {
    let root = temp_dir("ptn-native-string-offset-reads");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-offset-reads.php");
    let output = root.join("string-offset-reads-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"abcd\";\n\
var_dump($str[0]);\n\
var_dump($str[\"2\"]);\n\
var_dump($str[-1]);\n\
var_dump($str[4]);\n\
var_dump($str[-5]);\n\
var_dump($str[1][0]);\n\
var_dump($str[2][-2]);\n\
var_dump($str[true]);\n\
var_dump($str[false]);\n\
var_dump($str[null]);\n\
var_dump($str[1.8]);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"a\"\nstring(1) \"c\"\nstring(1) \"d\"\n\nWarning: Uninitialized string offset 4 in ptn on line 6\nstring(0) \"\"\n\nWarning: Uninitialized string offset -5 in ptn on line 7\nstring(0) \"\"\nstring(1) \"b\"\n\nWarning: Uninitialized string offset -2 in ptn on line 9\nstring(0) \"\"\n\nWarning: String offset cast occurred in ptn on line 10\nstring(1) \"b\"\n\nWarning: String offset cast occurred in ptn on line 11\nstring(1) \"a\"\n\nWarning: String offset cast occurred in ptn on line 12\nstring(1) \"a\"\n\nWarning: String offset cast occurred in ptn on line 13\nstring(1) \"b\"\n"
    );
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
fn compile_power_operator_to_native_binary() {
    let root = temp_dir("ptn-native-power-operator");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("power-operator.php");
    let output = root.join("power-operator-bin");
    fs::write(
        &input,
        "<?php $x = 2; $x **= 3; var_dump(-3 ** 2 === -9); var_dump((-3) ** 2 === 9); var_dump(2 ** 3 ** 2 === 512); var_dump((2 ** 3) ** 2 === 64); var_dump($x === 8);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
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
fn compile_goto_jump_phpt_shapes_to_native_binary() {
    let cases = [
        (
            "jump01",
            "<?php
$n = 1;
L1:
echo \"$n: ok\\n\";
$n++;
if ($n <= 3) goto L1;
?>",
            "1: ok\n2: ok\n3: ok\n",
        ),
        (
            "jump02",
            "<?php
$n = 1;
L1:
if ($n > 3) goto L2;
echo \"$n: ok\\n\";
$n++;
goto L1;
L2:
?>",
            "1: ok\n2: ok\n3: ok\n",
        ),
    ];

    for (name, source, expected) in cases {
        let root_name = format!("ptn-native-goto-{name}");
        let root = temp_dir(&root_name);
        fs::create_dir_all(&root).unwrap();
        let input = root.join(format!("{name}.php"));
        let output = root.join(format!("{name}-bin"));
        fs::write(&input, source).unwrap();

        compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

        let execution = Command::new(&output).output().unwrap();
        assert!(execution.status.success());
        assert_eq!(String::from_utf8(execution.stdout).unwrap(), expected);
        assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
    }
}

#[test]
fn compile_goto_from_loop_to_outer_label_to_native_binary() {
    let root = temp_dir("ptn-native-goto-jump04-loop-exit");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("jump04-loop-exit.php");
    let output = root.join("jump04-loop-exit-bin");
    fs::write(
        &input,
        "<?php
$s = \"X\";
echo \"1: ok\\n\";
L1: if ($s != \"X\") {
    echo \"4: ok\\n\";
} else {
    echo \"2: ok\\n\";
    while ($s != \"XXX\") {
        echo \"3: ok\\n\";
        $s .= \"X\";
        goto L1;
        echo \"bug\\n\";
    }
    echo \"bug\\n\";
}
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "1: ok\n2: ok\n3: ok\n4: ok\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_goto_inside_plain_blocks_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-goto-jump14-blocks");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("jump14.php");
    let output = root.join("jump14-bin");
    fs::write(
        &input,
        "<?php

goto A;

{
    B:
        goto C;
        return;
}

A:
    goto B;



{
    C:
    {
        print \"Done!\\n\";
    }
}

?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "Done!\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_return_statement_exits_script_to_native_binary() {
    let root = temp_dir("ptn-native-return-statement");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("return.php");
    let output = root.join("return-bin");
    fs::write(
        &input,
        "<?php echo \"before\\n\"; return var_dump(\"value\"); echo \"after\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "before\nstring(5) \"value\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn compile_break_level_exits_outer_control_target_to_native_binary() {
    let root = temp_dir("ptn-native-break-level-lang021-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("break-level-lang021.php");
    let output = root.join("break-level-lang021-bin");
    fs::write(
        &input,
        "<?php\n\
for ($i=0; $i<=5; $i++)\n\
{\n\
  echo \"i=$i\\n\";\n\
\n\
  switch($i) {\n\
    case 0:\n\
      echo \"In branch 0\\n\";\n\
      break;\n\
    case 1:\n\
      echo \"In branch 1\\n\";\n\
      break;\n\
    case 2:\n\
      echo \"In branch 2\\n\";\n\
      break;\n\
    case 3:\n\
      echo \"In branch 3\\n\";\n\
      break 2;\n\
    case 4:\n\
      echo \"In branch 4\\n\";\n\
      break;\n\
    default:\n\
      echo \"In default\\n\";\n\
      break;\n\
  }\n\
}\n\
echo \"hi\\n\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "i=0\nIn branch 0\ni=1\nIn branch 1\ni=2\nIn branch 2\ni=3\nIn branch 3\nhi\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_break_level_from_loop_inside_switch_to_native_binary() {
    let root = temp_dir("ptn-native-break-level-loop-inside-switch");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("break-level-loop-inside-switch.php");
    let output = root.join("break-level-loop-inside-switch-bin");
    fs::write(
        &input,
        "<?php switch (1) { case 1: $i = 0; while ($i < 5) { echo $i; if ($i == 2) break 2; $i++; } echo \"bad\"; break; } echo \":$i\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "012:2\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_single_statement_loop_bodies_to_native_binary() {
    let root = temp_dir("ptn-native-single-statement-loop-bodies");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("single-statement-loop-bodies.php");
    let output = root.join("single-statement-loop-bodies-bin");
    fs::write(
        &input,
        "<?php $i = 0; while ($i < 2) $i++; for (; $i < 4; $i++) echo $i; do print \":done\\n\"; while (false);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "23:done\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_unmatched_large_break_level_reports_source_line_to_native_binary() {
    let root = temp_dir("ptn-native-large-break-level-fatal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("large-break-level.php");
    let output = root.join("large-break-level-bin");
    fs::write(&input, "<?php\nfor(;;) break 2147483648;\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Cannot 'break' 2147483648 levels in {} on line 2\n",
            input.display()
        )
    );
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
        "int(-24)\nstring(8) \"8c90929a\"\n\nDeprecated: Implicit conversion from float 23.67 to int loses precision in ptn-generated-code on line 0\nint(-24)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_integer_operator_precision_deprecations_to_native_binary() {
    let root = temp_dir("ptn-native-int-operator-precision-deprecations");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("int-operator-precision-deprecations.php");
    let output = root.join("int-operator-precision-deprecations-bin");
    fs::write(
        &input,
        "<?php $var = 3; $var |= 1.5; var_dump($var); $var = 3; $var &= '1.5'; var_dump($var); $var = 9; $var %= 2.5; var_dump($var); $var = 9; $var %= '2.5'; var_dump($var);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Implicit conversion from float 1.5 to int loses precision in ptn-generated-code on line 0\nint(3)\n\nDeprecated: Implicit conversion from float-string \"1.5\" to int loses precision in ptn-generated-code on line 0\nint(1)\n\nDeprecated: Implicit conversion from float 2.5 to int loses precision in ptn-generated-code on line 0\nint(1)\n\nDeprecated: Implicit conversion from float-string \"2.5\" to int loses precision in ptn-generated-code on line 0\nint(1)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_intval_and_integer_operator_exponent_strings_to_native_binary() {
    let root = temp_dir("ptn-native-intval-exponent-strings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("intval-exponent-strings.php");
    let output = root.join("intval-exponent-strings-bin");
    fs::write(
        &input,
        "<?php var_dump((int)\"1.2345e9\"); var_dump(intval(\"-1.2345e9\")); var_dump(intval(\"ff\", 16)); var_dump(\" 1.2345e9  abc\" % PHP_INT_MAX); var_dump(\" -1.2345e9  abc\" | 0); var_dump(\"1.5abc\" | 0); var_dump(function_exists(\"intval\"), function_exists(\"INTVAL\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1234500000)\nint(-1234500000)\nint(255)\n\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\nint(1234500000)\n\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\nint(-1234500000)\n\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\n\nDeprecated: Implicit conversion from float-string \"1.5abc\" to int loses precision in ptn-generated-code on line 0\nint(1)\nbool(true)\nbool(true)\n"
    );
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
fn compile_non_canonical_boolean_cast_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-non-canonical-boolean-cast");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("non-canonical-boolean-cast.php");
    let output = root.join("non-canonical-boolean-cast-bin");
    fs::write(&input, "<?php\n\nvar_dump((boolean) 42);\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Deprecated: Non-canonical cast (boolean) is deprecated, use the (bool) cast instead in ptn on line 3\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_non_canonical_scalar_casts_phpt_shapes_to_native_binary() {
    let root = temp_dir("ptn-native-non-canonical-scalar-casts");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("non-canonical-scalar-casts.php");
    let output = root.join("non-canonical-scalar-casts-bin");
    fs::write(
        &input,
        "<?php\n\nvar_dump((integer) 42);\nvar_dump((double) 42);\nvar_dump((binary) 42);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Deprecated: Non-canonical cast (integer) is deprecated, use the (int) cast instead in ptn on line 3\nint(42)\n\nDeprecated: Non-canonical cast (double) is deprecated, use the (float) cast instead in ptn on line 4\nfloat(42)\n\nDeprecated: Non-canonical cast (binary) is deprecated, use the (string) cast instead in ptn on line 5\nstring(2) \"42\"\n"
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
    let unsupported_operator = parser::parse("<?php $name ??= 1;").unwrap_err();
    assert!(unsupported_operator
        .message
        .contains("unsupported PHP token '?'"));

    let unsupported_lvalue = parser::parse("<?php $items[0] += 1;").unwrap_err();
    assert!(unsupported_lvalue.message.contains("expected assignment"));
}

#[test]
fn var_dump_complex_edges_remain_unsupported_before_codegen() {
    for source in [
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
    assert!(support.contains("Array read expressions"));
    assert!(support.contains("String offset read expressions"));
    assert!(support.contains("Array element mutation"));
    assert!(support.contains("String offset writes/mutation"));
    assert!(support.contains("recursive arrays, objects, resources, references"));
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
