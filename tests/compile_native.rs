use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{
    AssignmentOp, BinaryOp, CastKind, Expr, IncDecOp, MagicConstantKind, Statement, StringPart,
    TypeHint, UnaryOp, UnsetTarget,
};
use ptn::lexer::{self, TokenKind};
use ptn::{compile_file, parser, CompileOptions, DiagnosticKind};

fn undefined_variable_warning(path: &Path, name: &str, line: usize) -> String {
    format!(
        "Warning: Undefined variable ${name} in {} on line {line}\n",
        path.display()
    )
}

fn undefined_variable_warnings(path: &Path, warnings: &[(&str, usize)]) -> String {
    let mut output = String::new();
    for (index, (name, line)) in warnings.iter().enumerate() {
        if index != 0 {
            output.push('\n');
        }
        output.push_str(&undefined_variable_warning(path, name, *line));
    }
    output
}

fn generated_c_static_function_body<'a>(c_source: &'a str, marker: &str) -> &'a str {
    let start = c_source
        .find(marker)
        .unwrap_or_else(|| panic!("generated runtime should contain {marker}"));
    let tail = &c_source[start..];
    let end = tail.find("\nstatic ").unwrap_or(tail.len());
    &tail[..end]
}

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
fn lexer_accepts_keyword_boolean_operators() {
    let tokens = lexer::lex("<?php true and false or true xor false").unwrap();
    assert!(matches!(tokens[2].kind, TokenKind::KeywordAnd));
    assert!(matches!(tokens[4].kind, TokenKind::KeywordOr));
    assert!(matches!(tokens[6].kind, TokenKind::KeywordXor));
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
fn parser_accepts_foreach_value_and_key_value_statements() {
    let program = parser::parse(
        "<?php foreach ($items as $value) { echo $value; } foreach ([1] as $key => $value) echo $key;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Foreach {
        iterable,
        key,
        value,
        body,
        ..
    } = &program.statements[0]
    else {
        panic!("expected value-only foreach statement");
    };
    assert!(matches!(iterable, Expr::Variable(name, _) if name == "items"));
    assert_eq!(key, &None);
    assert_eq!(value, "value");
    assert_eq!(body.len(), 1);

    let Statement::Foreach {
        key, value, body, ..
    } = &program.statements[1]
    else {
        panic!("expected key/value foreach statement");
    };
    assert_eq!(key.as_deref(), Some("key"));
    assert_eq!(value, "value");
    assert_eq!(body.len(), 1);
}

#[test]
fn parser_rejects_unsupported_foreach_bindings() {
    let by_ref = parser::parse("<?php foreach ($items as &$value) { echo $value; }").unwrap_err();
    assert_eq!(by_ref.message, "by-reference foreach is unsupported");

    let destructuring =
        parser::parse("<?php foreach ($items as [$value]) { echo $value; }").unwrap_err();
    assert_eq!(
        destructuring.message,
        "foreach destructuring is unsupported"
    );
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

    let const_delimiter = parser::parse("<?php\nconst FOO = \"BAR\"{0};").unwrap_err();
    assert_eq!(
        const_delimiter.message,
        "syntax error, unexpected token \"{\", expecting \",\" or \";\""
    );
    assert_eq!(const_delimiter.kind, DiagnosticKind::ParseError);
    let span = const_delimiter.span.unwrap();
    assert_eq!(span.line, 2);
    assert_eq!(span.column, 18);
}

#[test]
fn parser_rejects_unparenthesized_nested_ternaries() {
    let full = parser::parse("<?php\n\n1 ? 2 : 3 ? 4 : 5;").unwrap_err();
    assert_eq!(
        full.message,
        "Unparenthesized `a ? b : c ? d : e` is not supported. Use either `(a ? b : c) ? d : e` or `a ? b : (c ? d : e)`"
    );
    assert_eq!(full.kind, DiagnosticKind::Fatal);
    assert_eq!(full.span.unwrap().line, 3);

    let short_first = parser::parse("<?php\n\n1 ?: 2 ? 3 : 4;").unwrap_err();
    assert_eq!(
        short_first.message,
        "Unparenthesized `a ?: b ? c : d` is not supported. Use either `(a ?: b) ? c : d` or `a ?: (b ? c : d)`"
    );
    assert_eq!(short_first.kind, DiagnosticKind::Fatal);
    assert_eq!(short_first.span.unwrap().line, 3);

    let short_second = parser::parse("<?php\n\n1 ? 2 : 3 ?: 4;").unwrap_err();
    assert_eq!(
        short_second.message,
        "Unparenthesized `a ? b : c ?: d` is not supported. Use either `(a ? b : c) ?: d` or `a ? b : (c ?: d)`"
    );
    assert_eq!(short_second.kind, DiagnosticKind::Fatal);
    assert_eq!(short_second.span.unwrap().line, 3);
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
fn parser_accepts_user_function_declarations_and_returns() {
    let program = parser::parse(
        "<?php function add($left, $right) { $sum = $left + $right; return $sum; } echo add(2, 3);",
    )
    .unwrap();

    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.statements.len(), 1);
    let function = &program.functions[0];
    assert_eq!(function.name, "add");
    assert_eq!(function.parameters.len(), 2);
    assert_eq!(function.parameters[0].name, "left");
    assert_eq!(function.parameters[1].name, "right");
    assert_eq!(function.body.len(), 2);
    assert!(matches!(
        &function.body[1],
        Statement::Return { value: Some(_), .. }
    ));
}

#[test]
fn parser_preserves_user_function_declared_name_case() {
    let program =
        parser::parse("<?php function MixedCase() { return null; } mixedcase();").unwrap();

    assert_eq!(program.functions[0].name, "MixedCase");
}

#[test]
fn parser_accepts_null_parameter_and_return_type_hints() {
    let program =
        parser::parse("<?php function test(null $v): null { return $v; } var_dump(test(null));")
            .unwrap();

    let function = &program.functions[0];
    assert_eq!(function.return_type, Some(TypeHint::Null));
    assert_eq!(function.parameters[0].type_hint, Some(TypeHint::Null));
}

#[test]
fn parser_rejects_duplicate_user_function_declarations() {
    let error = parser::parse("<?php function same() {} function same() {}").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function same()");

    let error = parser::parse("<?php function Same() {} function same() {}").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function same()");
}

#[test]
fn parser_rejects_user_function_redeclaring_modeled_internal() {
    let error = parser::parse("<?php function STRLEN($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strlen()");

    let error = parser::parse("<?php function Substr($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function substr()");

    let error = parser::parse("<?php function CHUNK_SPLIT($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function chunk_split()");

    let error = parser::parse("<?php function Strip_Tags($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strip_tags()");

    let error =
        parser::parse("<?php function STR_STARTS_WITH($haystack, $needle) { return true; }")
            .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_starts_with()");

    let error = parser::parse("<?php function Str_Ends_With($haystack, $needle) { return true; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_ends_with()");

    let error = parser::parse("<?php function Quoted_Printable_Decode($value) { return $value; }")
        .unwrap_err();
    assert_eq!(
        error.message,
        "Cannot redeclare function quoted_printable_decode()"
    );

    let error = parser::parse("<?php function PhpVersion() { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function phpversion()");

    let error = parser::parse("<?php function PHP_SAPI_NAME() { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function php_sapi_name()");

    let error = parser::parse("<?php function IsSet($value) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function isset()");

    let error = parser::parse("<?php function Empty($value) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function empty()");

    let error = parser::parse("<?php function Abs($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function abs()");

    let error = parser::parse("<?php function Count($value) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function count()");

    let error = parser::parse("<?php function IntDiv($a, $b) { return $a; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function intdiv()");

    let error = parser::parse("<?php function ARRAY_KEY_EXISTS($key, $array) { return null; }")
        .unwrap_err();
    assert_eq!(
        error.message,
        "Cannot redeclare function array_key_exists()"
    );

    let error = parser::parse("<?php function End($array) { return $array; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function end()");

    let error = parser::parse("<?php function Prev($array) { return $array; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function prev()");

    let error = parser::parse("<?php function Print_R($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function print_r()");

    let error = parser::parse("<?php function array_values($array) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_values()");
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
fn parser_accepts_keyword_boolean_precedence() {
    let program = parser::parse(
        "<?php echo true || false and false, false or true && false, true xor false || false;",
    )
    .unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };

    let Expr::Binary {
        op: BinaryOp::And,
        left,
        ..
    } = &expressions[0]
    else {
        panic!("expected outer keyword and");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::Or,
            ..
        }
    ));

    let Expr::Binary {
        op: BinaryOp::Or,
        right,
        ..
    } = &expressions[1]
    else {
        panic!("expected outer keyword or");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::And,
            ..
        }
    ));

    let Expr::Binary {
        op: BinaryOp::Xor,
        right,
        ..
    } = &expressions[2]
    else {
        panic!("expected outer keyword xor");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Or,
            ..
        }
    ));
}

#[test]
fn parser_rejects_keyword_boolean_after_direct_assignment() {
    let error = parser::parse("<?php $result = true and false;").unwrap_err();
    assert!(error
        .message
        .contains("assignment expressions with keyword boolean operators are unsupported"));

    let compound = parser::parse("<?php $result += true xor false;").unwrap_err();
    assert!(compound
        .message
        .contains("assignment expressions with keyword boolean operators are unsupported"));

    parser::parse("<?php $result = (true and false);").unwrap();
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
    let program =
        parser::parse("<?php var_dump(array(1, \"2\" => 3, 4 => array(5)) <=> []);").unwrap();
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
fn parser_accepts_variable_root_array_assignment_and_unset() {
    let program = parser::parse(
        "<?php $items[null] = \"value\"; $items[] += 2; unset($items[null], $items);",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 3);

    let Statement::ArrayAssign {
        target, op, value, ..
    } = &program.statements[0]
    else {
        panic!("expected array assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::Assign);
    assert!(matches!(target.index.as_ref(), Some(Expr::Null(_))));
    assert!(matches!(value, Expr::String(value, _) if value == "value"));

    let Statement::ArrayAssign { target, op, .. } = &program.statements[1] else {
        panic!("expected array append compound assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::AddAssign);
    assert!(target.index.is_none());

    let Statement::Unset { targets, .. } = &program.statements[2] else {
        panic!("expected unset statement");
    };
    assert_eq!(targets.len(), 2);
    assert!(matches!(
        &targets[0],
        UnsetTarget::ArrayDim(target)
            if target.array == "items" && matches!(target.index.as_ref(), Some(Expr::Null(_)))
    ));
    assert!(matches!(
        &targets[1],
        UnsetTarget::Variable { name, .. } if name == "items"
    ));
}

#[test]
fn parser_accepts_long_array_literals_and_isset_empty_constructs() {
    let program = parser::parse(
        "<?php var_dump(isset($items[0], array('k' => 1)['k']), empty($items['missing']));",
    )
    .unwrap();
    let Statement::Call {
        name, arguments, ..
    } = &program.statements[0]
    else {
        panic!("expected var_dump statement");
    };
    assert_eq!(name, "var_dump");
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[0],
        Expr::Isset { targets, .. } if targets.len() == 2
            && matches!(&targets[1], Expr::ArrayAccess { array, .. } if matches!(array.as_ref(), Expr::Array { .. }))
    ));
    assert!(matches!(&arguments[1], Expr::Empty { .. }));
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
fn parser_accepts_supported_expression_statements() {
    let program =
        parser::parse("<?php Y; $value; ($value + 1); [1, 2][0]; strlen(\"abc\");").unwrap();
    assert_eq!(program.statements.len(), 5);
    assert!(matches!(
        &program.statements[0],
        Statement::Expression {
            expression: Expr::Constant(name, _),
            ..
        } if name == "Y"
    ));
    assert!(matches!(
        &program.statements[1],
        Statement::Expression {
            expression: Expr::Variable(name, _),
            ..
        } if name == "value"
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::Expression {
            expression: Expr::Grouped { .. },
            ..
        }
    ));
    assert!(matches!(
        &program.statements[3],
        Statement::Expression {
            expression: Expr::ArrayAccess { .. },
            ..
        }
    ));
    assert!(matches!(
        &program.statements[4],
        Statement::Call { name, .. } if name == "strlen"
    ));
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
fn parser_accepts_continue_statements_and_levels() {
    let program = parser::parse(
        "<?php for (;;) { while (true) { continue 2; } } do continue; while (false);",
    )
    .unwrap();

    let Statement::For { body, .. } = &program.statements[0] else {
        panic!("expected for statement");
    };
    let Statement::While {
        body: while_body, ..
    } = &body[0]
    else {
        panic!("expected nested while statement");
    };
    assert!(matches!(
        &while_body[0],
        Statement::Continue { level: 2, .. }
    ));

    let Statement::DoWhile { body, .. } = &program.statements[1] else {
        panic!("expected do while statement");
    };
    assert!(matches!(&body[0], Statement::Continue { level: 1, .. }));
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
fn phpc_renders_unexpected_const_terminator_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-unexpected-const-terminator");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unexpected-const-terminator.php");
    fs::write(&input, "<?php\nconst FOO_COMPILE_ERROR = \"BAR\"{0};\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: syntax error, unexpected token \"{{\", expecting \",\" or \";\" in {} on line 2\n",
            input.display()
        )
    );
}

#[test]
fn phpc_renders_unparenthesized_nested_ternary_as_php_fatal() {
    let root = temp_dir("ptn-phpc-nested-ternary-fatal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-ternary.php");
    fs::write(&input, "<?php\n\n1 ? 2 : 3 ? 4 : 5;\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Unparenthesized `a ? b : c ? d : e` is not supported. Use either `(a ? b : c) ? d : e` or `a ? b : (c ? d : e)` in {} on line 3\n",
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
fn compile_scalar_echo_keeps_direct_output_path_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-echo-direct-output");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-echo.php");
    let output = root.join("scalar-echo-bin");
    fs::write(
        &input,
        "<?php $word = \"runtime\"; echo \"literal \", $word, \" \", 123, \" \", 1.25, \" \", true, false, null, \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "literal runtime 123 1.25 1\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let echo_start = c_source
        .find("static PTN_UNUSED void ptn_echo(PtnValue value)")
        .expect("generated runtime should contain ptn_echo");
    let echo_tail = &c_source[echo_start..];
    let echo_end = echo_tail
        .find("\nint main(void)")
        .expect("echo-only generated runtime should omit internal-call helpers");
    let echo_body = &echo_tail[..echo_end];
    assert!(!echo_body.contains("ptn_value_to_string"));
    assert!(!c_source.contains("ptn_internal_var_dump"));
    assert!(!c_source.contains("ptn_call_internal"));
    assert!(echo_body.contains("case PTN_NULL:"));
    assert!(echo_body.contains("case PTN_BOOL:"));
    assert!(echo_body.contains("case PTN_INT:"));
    assert!(echo_body.contains("case PTN_FLOAT:"));
    assert!(echo_body.contains("case PTN_STRING:"));
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
fn compile_print_r_current_boxed_values_to_native_binary() {
    let root = temp_dir("ptn-native-print-r-current-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("print-r-current-values.php");
    let output = root.join("print-r-current-values-bin");
    fs::write(
        &input,
        r#"<?php
print_r([1, 2, 3]);
print_r(["x" => 1, "nested" => ["y" => 2], "" => null, true, false]);
var_dump(print_r(["a" => [1]], true));
var_dump(print_r(null, true), print_r(true, true), print_r(false, true), print_r(42, true), print_r(1.25, true), print_r("x", true));
var_dump(print_r("out"));
var_dump(function_exists("print_r"), function_exists("PRINT_R"));"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "Array\n",
            "(\n",
            "    [0] => 1\n",
            "    [1] => 2\n",
            "    [2] => 3\n",
            ")\n",
            "Array\n",
            "(\n",
            "    [x] => 1\n",
            "    [nested] => Array\n",
            "        (\n",
            "            [y] => 2\n",
            "        )\n",
            "\n",
            "    [] => \n",
            "    [0] => 1\n",
            "    [1] => \n",
            ")\n",
            "string(69) \"Array\n",
            "(\n",
            "    [a] => Array\n",
            "        (\n",
            "            [0] => 1\n",
            "        )\n",
            "\n",
            ")\n",
            "\"\n",
            "string(0) \"\"\n",
            "string(1) \"1\"\n",
            "string(0) \"\"\n",
            "string(2) \"42\"\n",
            "string(4) \"1.25\"\n",
            "string(1) \"x\"\n",
            "outbool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
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
        undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
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
fn compile_string_internals_use_direct_string_operand_fast_paths_to_native_binary() {
    let root = temp_dir("ptn-native-string-internal-direct-operands");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-internal-direct-operands.php");
    let output = root.join("string-internal-direct-operands-bin");
    fs::write(
        &input,
        "<?php\n\
echo strlen(\"abcdef\"), \" \", strcmp(\"abc\", \"abd\"), \" \", str_contains(\"abcdef\", \"cd\"), \" \", str_starts_with(\"abcdef\", \"ab\"), \" \", str_ends_with(\"abcdef\", \"ef\"), \"\\n\";\n\
echo str_rot13(\"abc\"), \" \", substr(\"abcdef\", 2, 3), \" \", bin2hex(\"Az\"), \" \", quotemeta(\"a.b\"), \" \", chunk_split(\"abcd\", 2, \"|\"), \"\\n\";\n\
echo strip_tags(\"<b>x</b>\"), \" \", quoted_printable_decode(\"=41\"), \" \", soundex(\"Robert\"), \" \", ord(\"A\"), \" \", bindec(\"101\"), \" \", hexdec(\"ff\"), \" \", octdec(\"10\"), \"\\n\";\n\
echo md5(\"\"), \" \", sha1(\"\"), \"\\n\";\n\
var_dump(strlen(12345), bin2hex(255), substr(12345, 1, 2));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "6 -1 1 1 1\nnop cde 417a a\\.b ab|cd|\n\
x A R163 65 5 255 8\n\
d41d8cd98f00b204e9800998ecf8427e da39a3ee5e6b4b0d3255bfef95601890afd80709\n\
int(5)\nstring(6) \"323535\"\nstring(2) \"23\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("case PTN_STRING:"));
    assert!(c_source.contains(
        "return ptn_string_operand_borrowed_len((const char *)value.as.string.data, value.as.string.len);"
    ));

    for function in [
        "ptn_internal_strlen",
        "ptn_internal_str_rot13",
        "ptn_internal_strcmp",
        "ptn_internal_str_contains",
        "ptn_internal_str_starts_with",
        "ptn_internal_str_ends_with",
        "ptn_internal_quotemeta",
        "ptn_internal_chunk_split",
        "ptn_internal_strip_tags",
        "ptn_internal_md5",
        "ptn_internal_sha1",
        "ptn_internal_substr",
        "ptn_internal_dirname",
        "ptn_internal_bin2hex",
        "ptn_internal_hex2bin",
        "ptn_internal_quoted_printable_decode",
        "ptn_internal_soundex",
        "ptn_internal_phpversion",
        "ptn_internal_bindec",
        "ptn_internal_hexdec",
        "ptn_internal_octdec",
        "ptn_internal_ord",
    ] {
        let marker = format!("static PtnValue {function}(");
        let body = generated_c_static_function_body(&c_source, &marker);
        assert!(
            body.contains("ptn_value_to_string_operand"),
            "{function} should use the direct string operand helper"
        );
        assert!(
            !body.contains("ptn_value_to_string(args"),
            "{function} should not convert direct argument expressions unconditionally"
        );
    }

    for expected_call in [
        "ptn_rot13_string(string.data, string.len)",
        "ptn_quotemeta_string(input.data, input.len, &output_len)",
        "ptn_strip_tags_string(input.data, input.len)",
        "ptn_dirname_string(path.data, path.len)",
        "ptn_quoted_printable_decode_string(input.data, input.len, &output_len)",
        "ptn_base_string_to_number(runtime, string.data, string.len, 2, 'b', line)",
        "ptn_base_string_to_number(runtime, string.data, string.len, 16, 'x', line)",
        "ptn_base_string_to_number(runtime, string.data, string.len, 8, 'o', line)",
    ] {
        assert!(
            c_source.contains(expected_call),
            "generated runtime should pass known operand lengths through {expected_call}"
        );
    }
    assert!(
        c_source.contains("input.data,\n        input.len,\n        (size_t)chunk_len_value,\n        ending.data,\n        ending.len,\n        &output_len"),
        "chunk_split helper should receive input and ending lengths"
    );

    for marker in [
        "static char *ptn_rot13_string(",
        "static char *ptn_quotemeta_string(",
        "static char *ptn_chunk_split_string(",
        "static char *ptn_strip_tags_string(",
        "static char *ptn_dirname_string(",
        "static char *ptn_quoted_printable_decode_string(",
        "static PtnValue ptn_base_string_to_number(",
    ] {
        let body = generated_c_static_function_body(&c_source, marker);
        assert!(
            !body.contains("strlen("),
            "{marker} should consume caller-provided lengths instead of rescanning"
        );
    }

    let soundex_body =
        generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_soundex(");
    assert!(
        soundex_body.contains("first < string.len") && soundex_body.contains("i < string.len"),
        "soundex should iterate using the known operand length"
    );
}

#[test]
fn compile_binary_safe_value_strings_to_native_binary() {
    let root = temp_dir("ptn-native-binary-safe-value-strings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("binary-safe-value-strings.php");
    let output = root.join("binary-safe-value-strings-bin");
    fs::write(
        &input,
        "<?php\n\
$s = \"a\" . chr(0) . \"b\";\n\
echo bin2hex(chr(0)), \"\\n\";\n\
echo strlen($s), \" \", bin2hex($s), \" \", ord($s[1]), \"\\n\";\n\
var_dump($s);\n\
echo $s;\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"00\n3 610062 0\nstring(3) \"a\0b\"\na\0b".to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_sha1_embedded_nul_input_and_raw_output_to_native_binary() {
    let root = temp_dir("ptn-native-sha1-binary-safe");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sha1-binary-safe.php");
    let output = root.join("sha1-binary-safe-bin");
    fs::write(
        &input,
        "<?php\n\
$s = \"a\" . chr(0) . \"b\";\n\
echo strlen($s), \" \", sha1($s), \"\\n\";\n\
echo bin2hex(sha1($s, true)), \"\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "3 4a3dec2d1f8245280855c42db0ee4239f917fdb8\n4a3dec2d1f8245280855c42db0ee4239f917fdb8\n"
    );
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
fn compile_intdiv_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-intdiv-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("intdiv-function.php");
    let output = root.join("intdiv-function-bin");
    fs::write(
        &input,
        "<?php var_dump(intdiv(3, 2), intdiv(-3, 2), intdiv(3, -2), intdiv(-3, -2), intdiv(PHP_INT_MAX, PHP_INT_MAX), intdiv(PHP_INT_MIN, PHP_INT_MIN), intdiv(\"9\", \"2\"), function_exists(\"intdiv\"), function_exists(\"INTDIV\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(-1)\nint(-1)\nint(1)\nint(1)\nint(1)\nint(4)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_intdiv_float_precision_diagnostic_to_native_binary() {
    let root = temp_dir("ptn-native-intdiv-float-precision");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("intdiv-float-precision.php");
    let output = root.join("intdiv-float-precision-bin");
    fs::write(&input, "<?php var_dump(intdiv(5.9, 2));").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Implicit conversion from float 5.9 to int loses precision in ptn-generated-code on line 0\nint(2)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn var_dump_float_exponents_use_php_spelling_in_native_binary() {
    let root = temp_dir("ptn-native-var-dump-float-exponents");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-dump-float-exponents.php");
    let output = root.join("var-dump-float-exponents-bin");
    fs::write(
        &input,
        "<?php var_dump(-9.22337203900226E+18); var_dump(1.4757395258967642E+19); var_dump(1.2e-5);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(-9.22337203900226E+18)\nfloat(1.4757395258967642E+19)\nfloat(1.2E-5)\n"
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
fn compile_chunk_split_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-chunk-split-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chunk-split-basic.php");
    let output = root.join("chunk-split-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing chunk_split() : basic functionality ***\\n\";\n\
$str = 'Testing';\n\
$chunklen = 2;\n\
$ending = '##';\n\
echo \"-- Testing chunk_split() with all possible arguments --\\n\";\n\
var_dump(chunk_split($str, $chunklen, $ending));\n\
echo \"-- Testing chunk_split() with default ending string --\\n\";\n\
var_dump(chunk_split($str, $chunklen));\n\
echo \"-- Testing chunk_split() with default chunklen and ending string --\\n\";\n\
var_dump(chunk_split($str));\n\
echo \"Done\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing chunk_split() : basic functionality ***\n-- Testing chunk_split() with all possible arguments --\nstring(15) \"Te##st##in##g##\"\n-- Testing chunk_split() with default ending string --\nstring(15) \"Te\r\nst\r\nin\r\ng\r\n\"\n-- Testing chunk_split() with default chunklen and ending string --\nstring(9) \"Testing\r\n\"\nDone"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chunk_split_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-chunk-split-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chunk-split-registry.php");
    let output = root.join("chunk-split-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(chunk_split(12345, 2, \".\"), chunk_split(\"abc\", \"2\", true), function_exists(\"chunk_split\"), function_exists(\"CHUNK_SPLIT\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(8) \"12.34.5.\"\nstring(5) \"ab1c1\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_strip_tags_bug70720_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-strip-tags-bug70720-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strip-tags-bug70720.php");
    let output = root.join("strip-tags-bug70720-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(strip_tags('<?php $dom->test(); ?> this is a test'));\n\
var_dump(strip_tags('<?php $xml->test(); ?> this is a test'));\n\
var_dump(strip_tags('<?xml $xml->test(); ?> this is a test'));\n\
var_dump(strip_tags(\"<span class=sf-dump-> this is a test</span>\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(15) \" this is a test\"\nstring(15) \" this is a test\"\nstring(15) \" this is a test\"\nstring(15) \" this is a test\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_strip_tags_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-strip-tags-registry-and-scalar-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strip-tags-registry.php");
    let output = root.join("strip-tags-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(strip_tags(12345), strip_tags(true), strip_tags(\"<b>x</b><i>y</i>\"), strip_tags(\"a < b\"), strip_tags(\"a<!-- c -->b\"), strip_tags(\"<% echo hi %> ok\"), function_exists(\"strip_tags\"), function_exists(\"STRIP_TAGS\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"12345\"\nstring(1) \"1\"\nstring(2) \"xy\"\nstring(5) \"a < b\"\nstring(2) \"ab\"\nstring(3) \" ok\"\nbool(true)\nbool(true)\n"
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
fn compile_str_starts_and_ends_with_phpt_shapes_to_native_binary() {
    let root = temp_dir("ptn-native-str-starts-ends-with-phpt-shapes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-starts-ends-with.php");
    let output = root.join("str-starts-ends-with-bin");
    fs::write(
        &input,
        "<?php\n\
$testStr = \"beginningMiddleEnd\";\n\
var_dump(str_starts_with($testStr, \"beginning\"));\n\
var_dump(str_starts_with($testStr, \"Beginning\"));\n\
var_dump(str_starts_with($testStr, \"eginning\"));\n\
var_dump(str_starts_with($testStr, $testStr));\n\
var_dump(str_starts_with($testStr, $testStr.$testStr));\n\
var_dump(str_starts_with($testStr, \"\"));\n\
var_dump(str_starts_with(\"\", \"\"));\n\
var_dump(str_starts_with(\"\", \" \"));\n\
var_dump(str_starts_with($testStr, \"\\x00\"));\n\
var_dump(str_starts_with(\"\\x00\", \"\"));\n\
var_dump(str_starts_with(\"\\x00\", \"\\x00\"));\n\
var_dump(str_starts_with(\"\\x00a\", \"\\x00\"));\n\
var_dump(str_starts_with(\"a\\x00bc\", \"a\\x00b\"));\n\
var_dump(str_starts_with(\"a\\x00b\", \"a\\x00d\"));\n\
var_dump(str_starts_with(\"a\\x00b\", \"z\\x00b\"));\n\
var_dump(str_starts_with(\"a\", \"a\\x00\"));\n\
var_dump(str_starts_with(\"a\", \"\\x00a\"));\n\
var_dump(str_ends_with($testStr, \"End\"));\n\
var_dump(str_ends_with($testStr, \"end\"));\n\
var_dump(str_ends_with($testStr, \"en\"));\n\
var_dump(str_ends_with($testStr, $testStr));\n\
var_dump(str_ends_with($testStr, $testStr.$testStr));\n\
var_dump(str_ends_with($testStr, \"\"));\n\
var_dump(str_ends_with(\"\", \"\"));\n\
var_dump(str_ends_with(\"\", \" \"));\n\
var_dump(str_ends_with($testStr, \"\\x00\"));\n\
var_dump(str_ends_with(\"\\x00\", \"\"));\n\
var_dump(str_ends_with(\"\\x00\", \"\\x00\"));\n\
var_dump(str_ends_with(\"a\\x00\", \"\\x00\"));\n\
var_dump(str_ends_with(\"ab\\x00c\", \"b\\x00c\"));\n\
var_dump(str_ends_with(\"a\\x00b\", \"d\\x00b\"));\n\
var_dump(str_ends_with(\"a\\x00b\", \"a\\x00z\"));\n\
var_dump(str_ends_with(\"a\", \"\\x00a\"));\n\
var_dump(str_ends_with(\"a\", \"a\\x00\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_str_starts_and_ends_with_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-str-starts-ends-with-registry");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-starts-ends-with-registry.php");
    let output = root.join("str-starts-ends-with-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(str_starts_with(12345, \"12\"), str_starts_with(true, \"1\"), str_ends_with(12345, 45), str_ends_with(false, \"\"), function_exists(\"str_starts_with\"), function_exists(\"STR_ENDS_WITH\"));",
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
fn compile_quoted_printable_decode_phpt_shapes_to_native_binary() {
    let root = temp_dir("ptn-native-quoted-printable-decode-phpt-shapes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("quoted-printable-decode.php");
    let output = root.join("quoted-printable-decode-bin");
    fs::write(
        &input,
        "<?php\n\
echo bin2hex(quoted_printable_decode(\"=FAwow-factor=C1=d0=D5=DD=C5=CE=CE=D9=C5=0A=\n\
=20=D4=cf=D2=C7=CF=D7=D9=C5=\n\
=20=\n\
=D0=\n\
=D2=CF=C5=CB=D4=D9\")), \"\\n\";\n\
echo bin2hex(quoted_printable_decode(\"=FAwow-factor=C1=D0=D5=DD=C5=CE=CE=D9=C5=0A=\n\
=20=D4=CF=D2=C7=CF=D7=D9=C5=\n\
=20=\n\
=D0=\n\
=D2=CF=C5=CB=D4=D9\")), \"\\n\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "fa776f772d666163746f72c1d0d5ddc5ceced9c50a20d4cfd2c7cfd7d9c520d0d2cfc5cbd4d9\n\
fa776f772d666163746f72c1d0d5ddc5ceced9c50a20d4cfd2c7cfd7d9c520d0d2cfc5cbd4d9\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_quoted_printable_decode_registry_and_scalar_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-quoted-printable-decode-registry");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("quoted-printable-decode-registry.php");
    let output = root.join("quoted-printable-decode-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(quoted_printable_decode(\"Hello=20World=21\"), quoted_printable_decode(true), quoted_printable_decode(123), function_exists(\"quoted_printable_decode\"), function_exists(\"QUOTED_PRINTABLE_DECODE\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(12) \"Hello World!\"\nstring(1) \"1\"\nstring(3) \"123\"\nbool(true)\nbool(true)\n"
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
        undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
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
fn compile_user_function_parameters_and_return_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-parameters-return");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-functions.php");
    let output = root.join("user-functions-bin");
    fs::write(
        &input,
        "<?php function add($left, $right) { $sum = $left + $right; return $sum; } function label($value) { return \"value=\" . $value; } echo label(add(2, 3)), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "value=5\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_extra_arguments_are_accepted_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-extra-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-extra-arguments.php");
    let output = root.join("user-function-extra-arguments-bin");
    fs::write(
        &input,
        "<?php function first($value) { return $value; } function zero() { return \"zero\"; } echo first(\"kept\", \"extra\"), \" \", zero(\"ignored\"), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "kept zero\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_func_introspection_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-func-introspection");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-func-introspection.php");
    let output = root.join("user-function-func-introspection-bin");
    fs::write(
        &input,
        "<?php function inspect($left, $right) { $left = \"changed\"; unset($right); var_dump(func_num_args(), func_get_arg(0), func_get_arg(1), func_get_arg(2), func_get_args()); } inspect(\"original\", \"gone\", \"extra\");",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(3)\nstring(7) \"changed\"\nNULL\nstring(5) \"extra\"\narray(3) {\n  [0]=>\n  string(7) \"changed\"\n  [1]=>\n  NULL\n  [2]=>\n  string(5) \"extra\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_zero_parameter_user_function_func_args_to_native_binary() {
    let root = temp_dir("ptn-native-zero-parameter-user-function-func-args");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("zero-parameter-user-function-func-args.php");
    let output = root.join("zero-parameter-user-function-func-args-bin");
    fs::write(
        &input,
        "<?php function inspect() { var_dump(func_num_args(), func_get_args()); } inspect(1, \"two\");",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(2)\narray(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  string(3) \"two\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_func_introspection_registry_and_global_errors_to_native_binary() {
    let root = temp_dir("ptn-native-func-introspection-global-errors");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("func-introspection-global-errors.php");
    let output = root.join("func-introspection-global-errors-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"func_num_args\"), function_exists(\"FUNC_GET_ARG\"), function_exists(\"func_get_args\")); try { func_num_args(); } catch (Error $e) { echo $e->getMessage(), \"\\n\"; } try { func_get_arg(0); } catch (Error $e) { echo $e->getMessage(), \"\\n\"; } try { func_get_args(); } catch (Error $e) { echo $e->getMessage(), \"\\n\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nfunc_num_args() must be called from a function context\nfunc_get_arg() cannot be called from the global scope\nfunc_get_args() cannot be called from the global scope\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_locals_do_not_overwrite_top_level_variables() {
    let root = temp_dir("ptn-native-user-function-local-scope");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-local-scope.php");
    let output = root.join("user-function-local-scope-bin");
    fs::write(
        &input,
        "<?php $value = \"global\"; function change($value) { $value = $value . \"-local\"; return $value; } echo change(\"arg\"), \" \", $value, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "arg-local global\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_early_return_and_implicit_null_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-return-flow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-return-flow.php");
    let output = root.join("user-function-return-flow-bin");
    fs::write(
        &input,
        "<?php function choose($value) { if ($value) { return \"yes\"; } echo \"fallback \"; return \"no\"; } function nothing() { echo \"side \"; } echo choose(true), \" \", choose(false), \" \"; var_dump(nothing());",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "yes fallback no side NULL\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_recursive_user_function_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-recursion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-recursion.php");
    let output = root.join("user-function-recursion-bin");
    fs::write(
        &input,
        "<?php function fact($n) { if ($n <= 1) { return 1; } return $n * fact($n - 1); } echo fact(5), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "120\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_calls_use_direct_generated_path_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-direct-call-path");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-direct-call-path.php");
    let output = root.join("user-function-direct-call-path-bin");
    fs::write(
        &input,
        "<?php const BASE = 7; function step($value) { return $value + BASE; } function apply($value) { return STEP($value); } echo apply(5), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "12\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(!c_source.contains("ptn_runtime_import_constants"));
    assert!(c_source.contains("ptn_runtime_init_function_frame(&runtime, caller_runtime);"));
    assert!(c_source.contains("ptn_user_function_0(&runtime, 1,"));

    let main_start = c_source.find("\nint main(void)").unwrap();
    let main_body = &c_source[main_start..];
    assert!(main_body.contains("ptn_user_function_1(&runtime, 1,"));
    assert!(!main_body.contains("ptn_call_function(&runtime, \"apply\""));
    assert!(!c_source.contains("ptn_call_internal"));
    assert!(!c_source.contains("ptn_internal_var_dump"));
}

#[test]
fn compile_user_function_reads_global_const_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-global-const");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-global-const.php");
    let output = root.join("user-function-global-const-bin");
    fs::write(
        &input,
        "<?php const BASE = 4; function scale($value) { return $value * BASE; } echo scale(3), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "12\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_define_updates_shared_constant_table_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-shared-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-shared-constants.php");
    let output = root.join("user-function-shared-constants-bin");
    fs::write(
        &input,
        "<?php function set_constant() { define(\"FROM_FUNCTION\", 13); return constant(\"FROM_FUNCTION\"); } echo set_constant(), \" \", constant(\"FROM_FUNCTION\"), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "13 13\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_exists_registry_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-exists");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-exists.php");
    let output = root.join("user-function-exists-bin");
    fs::write(
        &input,
        "<?php function local() { return null; } var_dump(function_exists(\"local\"), function_exists(\"LOCAL\"), function_exists(\"strlen\"), function_exists(\"missing\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_null_type_errors_with_direct_call_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-null-type-error-direct");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-null-type-error.php");
    let output = root.join("user-function-null-type-error-bin");
    fs::write(
        &input,
        "<?php function test(null $v): null { return $v; } var_dump(test(1));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: test() argument $v must be of type null\n"
    );
}

#[test]
fn compile_null_typed_user_function_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-null-type");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-null-type.php");
    let output = root.join("user-function-null-type-bin");
    fs::write(
        &input,
        "<?php function test(null $v): null { return $v; } var_dump(test(null));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "NULL\n");
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
        "<?php var_dump(gettype(null), gettype(true), gettype(42), gettype(1.5), gettype(\"x\")); var_dump(is_null(null), is_bool(false), is_int(1), is_integer(1), is_long(1), is_float(1.5), is_double(1.5), is_string(\"x\"), is_scalar(\"x\"), is_scalar(null), is_array(null), is_array(42), is_array(\"x\"), is_float('-.1' * 2));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"NULL\"\nstring(7) \"boolean\"\nstring(7) \"integer\"\nstring(6) \"double\"\nstring(6) \"string\"\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\n"
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
fn compile_user_function_magic_constants_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-magic-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("function-magic-constants.php");
    let output = root.join("function-magic-constants-bin");
    fs::write(
        &input,
        "<?php\n\
function MixedCase() { var_dump(__LINE__, __FILE__, __DIR__, __FUNCTION__, __METHOD__, __CLASS__, __TRAIT__, __NAMESPACE__); }\n\
var_dump(__FUNCTION__, __METHOD__);\n\
mixedcase();\n\
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
            "string(0) \"\"\nstring(0) \"\"\nint(2)\nstring({}) \"{}\"\nstring({}) \"{}\"\nstring(9) \"MixedCase\"\nstring(9) \"MixedCase\"\nstring(0) \"\"\nstring(0) \"\"\nstring(0) \"\"\n",
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
fn compile_php_sapi_name_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-php-sapi-name-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("php-sapi-name.php");
    let output = root.join("php-sapi-name-bin");
    fs::write(
        &input,
        "<?php\n\
\n\
var_dump(php_sapi_name());\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(3) \"cli\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_phpversion_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-phpversion-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("phpversion.php");
    let output = root.join("phpversion-bin");
    fs::write(
        &input,
        "<?php\n\
\n\
print phpversion();\n\
print \"\\n\";\n\
print phpversion('standard');\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "8.4.0\n8.4.0");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_versioning_registry_and_unknown_extension_to_native_binary() {
    let root = temp_dir("ptn-native-versioning-registry");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("versioning-registry.php");
    let output = root.join("versioning-registry-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"php_sapi_name\"), function_exists(\"PHPVERSION\"), phpversion(\"STANDARD\"), phpversion(\"missing_extension\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nstring(5) \"8.4.0\"\nbool(false)\n"
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
fn compile_internal_function_registry_lookup_edges_to_native_binary() {
    let root = temp_dir("ptn-native-internal-registry-lookup-edges");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("internal-registry-lookup-edges.php");
    let output = root.join("internal-registry-lookup-edges-bin");
    fs::write(
        &input,
        "<?php var_dump(function_exists(\"ABS\"), function_exists(\"array_key_exists\"), function_exists(\"SUBSTR\"), function_exists(\"VAR_DUMP\"), function_exists(\"missing_internal\")); echo abs(-5), \" \", strlen(\"abc\"), \" \", substr(\"abcdef\", 2, 3), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\n5 3 cde\n"
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
fn compile_define_then_const_duplicate_preserves_original_constant() {
    let root = temp_dir("ptn-native-define-then-const-duplicate");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("define-then-const-duplicate.php");
    let output = root.join("define-then-const-duplicate-bin");
    fs::write(
        &input,
        "<?php\n\
define(\"a\", 2);\n\
const a = 1;\n\
if (defined(\"a\")) {\n\
    print a;\n\
}\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: Constant a already defined, this will be an error in PHP 9 in ptn on line 3\n2"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_duplicate_const_declarations_warn_and_keep_first_value() {
    let root = temp_dir("ptn-native-duplicate-const-declarations");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("duplicate-const-declarations.php");
    let output = root.join("duplicate-const-declarations-bin");
    fs::write(&input, "<?php\nconst C = 1, C = 2;\nvar_dump(C);\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: Constant C already defined, this will be an error in PHP 9 in ptn on line 2\nint(1)\n"
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
fn compile_keyword_boolean_ops_to_native_binary() {
    let root = temp_dir("ptn-native-keyword-boolean-ops");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("keyword-boolean.php");
    let output = root.join("keyword-boolean-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(true and false);\n\
var_dump(false or true);\n\
var_dump(true xor false);\n\
var_dump(true xor true);\n\
var_dump(true || false and false);\n\
var_dump(false or true && false);\n\
var_dump(false and $short_circuit_and);\n\
var_dump(true or $short_circuit_or);\n\
var_dump($left xor $right);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        undefined_variable_warnings(&input, &[("left", 10), ("right", 10)])
    );
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
var_dump(is_scalar([]));\n\
var_dump(is_array([]));\n\
var_dump(is_array([1 => \"a\", 0 => \"b\"]));\n\
var_dump(function_exists(\"is_array\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(-1)\nint(-1)\nint(-1)\nint(1)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\narray(2) {\n  [0]=>\n  int(9)\n  [\"\"]=>\n  int(8)\n}\nstring(5) \"array\"\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
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
fn compile_numeric_string_array_key_normalization_to_native_binary() {
    let root = temp_dir("ptn-native-numeric-string-array-key-normalization");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("numeric-string-array-key-normalization.php");
    let output = root.join("numeric-string-array-key-normalization-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\n\
    \"8\" => \"literal-int\",\n\
    \"08\" => \"literal-leading-zero\",\n\
    \"+8\" => \"literal-plus\",\n\
    \"8.0\" => \"literal-float\",\n\
    \"-8\" => \"literal-negative\",\n\
    \"-0\" => \"literal-minus-zero\",\n\
    \"alpha\" => \"literal-alpha\",\n\
];\n\
var_dump($items);\n\
var_dump($items[8]);\n\
var_dump($items[\"8\"]);\n\
var_dump($items[\"08\"]);\n\
var_dump($items[\"+8\"]);\n\
var_dump($items[\"8.0\"]);\n\
var_dump($items[-8]);\n\
var_dump($items[\"-8\"]);\n\
var_dump($items[\"-0\"]);\n\
var_dump($items[\"alpha\"]);\n\
var_dump(array_key_exists(8, $items));\n\
var_dump(array_key_exists(\"08\", $items));\n\
var_dump(array_key_exists(-8, $items));\n\
var_dump(array_key_exists(\"-8\", $items));\n\
var_dump(isset($items[\"+8\"]), empty($items[\"missing\"]));\n\
$items[\"9\"] = \"write-nine\";\n\
var_dump($items[9]);\n\
$items[9] = \"write-nine-replaced\";\n\
var_dump($items[\"9\"]);\n\
$items[\"09\"] = \"write-zero-nine\";\n\
var_dump($items[9]);\n\
var_dump($items[\"09\"]);\n\
$items[\"-9\"] = \"write-negative-nine\";\n\
var_dump($items[-9]);\n\
unset($items[\"9\"]);\n\
var_dump(array_key_exists(9, $items));\n\
var_dump(array_key_exists(\"09\", $items));\n\
foreach ($items as $key => $value) {\n\
    echo gettype($key), \":\", $key, \"=\", $value, \"\\n\";\n\
}",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(7) {\n  [8]=>\n  string(11) \"literal-int\"\n  [\"08\"]=>\n  string(20) \"literal-leading-zero\"\n  [\"+8\"]=>\n  string(12) \"literal-plus\"\n  [\"8.0\"]=>\n  string(13) \"literal-float\"\n  [-8]=>\n  string(16) \"literal-negative\"\n  [\"-0\"]=>\n  string(18) \"literal-minus-zero\"\n  [\"alpha\"]=>\n  string(13) \"literal-alpha\"\n}\nstring(11) \"literal-int\"\nstring(11) \"literal-int\"\nstring(20) \"literal-leading-zero\"\nstring(12) \"literal-plus\"\nstring(13) \"literal-float\"\nstring(16) \"literal-negative\"\nstring(16) \"literal-negative\"\nstring(18) \"literal-minus-zero\"\nstring(13) \"literal-alpha\"\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nstring(10) \"write-nine\"\nstring(19) \"write-nine-replaced\"\nstring(19) \"write-nine-replaced\"\nstring(15) \"write-zero-nine\"\nstring(19) \"write-negative-nine\"\nbool(false)\nbool(true)\ninteger:8=literal-int\nstring:08=literal-leading-zero\nstring:+8=literal-plus\nstring:8.0=literal-float\ninteger:-8=literal-negative\nstring:-0=literal-minus-zero\nstring:alpha=literal-alpha\nstring:09=write-zero-nine\ninteger:-9=write-negative-nine\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_legacy_array_count_and_abs_to_native_binary() {
    let root = temp_dir("ptn-native-legacy-array-count-abs");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("legacy-array-count-abs.php");
    let output = root.join("legacy-array-count-abs-bin");
    fs::write(
        &input,
        "<?php\n\
$values = array(23, -23, \"23.45\", null, true, false);\n\
for ($i = 0; $i < count($values); $i++) {\n\
    var_dump(abs($values[$i]));\n\
}\n\
var_dump(count(array(\"x\" => 1, 2)));\n\
var_dump(function_exists(\"COUNT\"), function_exists(\"abs\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(23)\nint(23)\nfloat(23.45)\n\nDeprecated: abs(): Passing null to parameter #1 ($num) of type int|float is deprecated in ptn on line 4\nint(0)\nint(1)\nint(0)\nint(2)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_count_type_errors_are_catchable_to_native_binary() {
    let root = temp_dir("ptn-native-count-type-errors-catchable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("count-type-errors-catchable.php");
    let output = root.join("count-type-errors-catchable-bin");
    fs::write(
        &input,
        "<?php\n\
$values = [[], [1, 2], null, false, true, 42, 1.25, \"abc\"];\n\
foreach ($values as $value) {\n\
    try {\n\
        var_dump(count($value));\n\
    } catch (\\TypeError $e) {\n\
        echo $e->getMessage(), \"\\n\";\n\
    }\n\
}\n\
echo \"after\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(0)\nint(2)\ncount(): Argument #1 ($value) must be of type Countable|array, null given\ncount(): Argument #1 ($value) must be of type Countable|array, false given\ncount(): Argument #1 ($value) must be of type Countable|array, true given\ncount(): Argument #1 ($value) must be of type Countable|array, int given\ncount(): Argument #1 ($value) must be of type Countable|array, float given\ncount(): Argument #1 ($value) must be of type Countable|array, string given\nafter\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_uncaught_count_type_error_fatals() {
    let root = temp_dir("ptn-native-uncaught-count-type-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("uncaught-count-type-error.php");
    let output = root.join("uncaught-count-type-error-bin");
    fs::write(
        &input,
        "<?php\n\
count(false);\n\
echo \"unreached\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: count(): Argument #1 ($value) must be of type Countable|array, false given\n"
    );
}

#[test]
fn compile_foreach_value_loop_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-values.php");
    let output = root.join("foreach-values-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [1, 2, 3];\n\
$total = 0;\n\
foreach ($items as $value) {\n\
    echo $value;\n\
    $total += $value;\n\
}\n\
echo \":$total\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "123:6\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_key_value_loop_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-key-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-key-values.php");
    let output = root.join("foreach-key-values-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"a\" => 2, \"b\" => 3, 5];\n\
$total = 0;\n\
foreach ($items as $key => $value) {\n\
    echo $key, \":\", $value, \"\\n\";\n\
    $total += $value;\n\
}\n\
echo \"total=\", $total, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "a:2\nb:3\n0:5\ntotal=10\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_evaluates_iterable_once_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-evaluate-once");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-evaluate-once.php");
    let output = root.join("foreach-evaluate-once-bin");
    fs::write(
        &input,
        "<?php foreach ([var_dump(\"make\"), 1, 2] as $value) { echo \"v\"; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"make\"\nvvv\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_break_and_continue_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-break-continue");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-break-continue.php");
    let output = root.join("foreach-break-continue-bin");
    fs::write(
        &input,
        "<?php foreach ([1, 2, 3, 4] as $value) { if ($value == 2) continue; if ($value == 4) break; echo $value; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "13\n");
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
fn compile_array_offset_assignment_and_unset_to_native_binary() {
    let root = temp_dir("ptn-native-array-offset-assignment-unset");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-offset-assignment-unset.php");
    let output = root.join("array-offset-assignment-unset-bin");
    fs::write(
        &input,
        "<?php\n\
$arr = ['foo' => 'bar', '' => 'baz'];\n\
echo $arr[null] . \"\\n\";\n\
$arr[null] = 'new_value';\n\
echo $arr[''] . \"\\n\";\n\
var_dump(isset($arr[null]));\n\
unset($arr[null]);\n\
var_dump(isset($arr['']));\n\
$items = [];\n\
$items[\"7\"] = \"seven\";\n\
var_dump($items[7]);\n\
unset($items[7]);\n\
var_dump(isset($items[\"7\"]));\n\
$items[] = \"appended\";\n\
var_dump($items[8]);\n\
$items[] += 2;\n\
var_dump($items[9]);\n\
$items[8] .= \"-tail\";\n\
var_dump($items[8]);\n\
$items[10] += 5;\n\
var_dump($items[10]);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 3\nbaz\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 4\nnew_value\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 6\nbool(true)\nbool(false)\nstring(5) \"seven\"\nbool(false)\nstring(8) \"appended\"\nint(2)\nstring(13) \"appended-tail\"\n\nWarning: Undefined array key 10 in ptn on line 20\nint(5)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_offset_compound_assignment_undef_to_native_binary() {
    let root = temp_dir("ptn-native-array-offset-compound-undef");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-offset-compound-undef.php");
    let output = root.join("array-offset-compound-undef-bin");
    fs::write(
        &input,
        "<?php\n\
$items[0] += 2;\n\
var_dump($items[0]);\n\
$append[] .= \"x\";\n\
var_dump($append[0]);\n\
$a[$b] += 1;\n\
var_dump($a);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nWarning: Undefined array key 0 in ptn on line 2\nint(2)\nstring(1) \"x\"\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 6\n\nWarning: Undefined array key \"\" in ptn on line 6\narray(1) {\n  [\"\"]=>\n  int(1)\n}\n"
    );
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        undefined_variable_warnings(&input, &[("items", 2), ("append", 4), ("a", 6), ("b", 6)])
    );
}

#[test]
fn compile_isset_empty_offsets_to_native_binary() {
    let root = temp_dir("ptn-native-isset-empty-offsets");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("isset-empty-offsets.php");
    let output = root.join("isset-empty-offsets-bin");
    fs::write(
        &input,
        "<?php\n\
$items = array(\"empty\" => \"0\", \"truthy\" => 1, \"null\" => null, \"nested\" => [\"leaf\" => \"\"]);\n\
var_dump(isset($items[\"truthy\"]));\n\
var_dump(isset($items[\"null\"]));\n\
var_dump(isset($items[\"missing\"]));\n\
var_dump(isset($missing));\n\
var_dump(isset($items[\"nested\"][\"leaf\"]));\n\
var_dump(empty($items[\"empty\"]));\n\
var_dump(empty($items[\"truthy\"]));\n\
var_dump(empty($items[\"missing\"]));\n\
var_dump(empty($missing));\n\
$string = \"foobar\";\n\
var_dump(isset($string[0][0][0][0]));\n\
var_dump(isset($string[\"foo\"]));\n\
var_dump(empty($string[\"foo\"]));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_key_exists_to_native_binary() {
    let root = temp_dir("ptn-native-array-key-exists");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-key-exists.php");
    let output = root.join("array-key-exists-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"foo\" => \"bar\", \"\" => \"empty\", \"0\" => \"zero\", \"n\" => null];\n\
var_dump(array_key_exists(\"foo\", $items));\n\
var_dump(array_key_exists(\"missing\", $items));\n\
var_dump(array_key_exists(null, $items));\n\
var_dump(array_key_exists(\"\", $items));\n\
var_dump(array_key_exists(\"0\", $items));\n\
var_dump(array_key_exists(\"n\", $items));\n\
var_dump(function_exists(\"array_key_exists\"), function_exists(\"ARRAY_KEY_EXISTS\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nDeprecated: Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead in ptn on line 5\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_values_to_native_binary() {
    let root = temp_dir("ptn-native-array-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-values.php");
    let output = root.join("array-values-bin");
    fs::write(
        &input,
        "<?php\n\
$items = array('zero', 'one', 'two', 'three' => 3, 10 => 'ten');\n\
$values = array_values($items);\n\
var_dump($values);\n\
var_dump($items);\n\
var_dump(array_values([]));\n\
var_dump(function_exists(\"array_values\"), function_exists(\"ARRAY_VALUES\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(5) {\n  [0]=>\n  string(4) \"zero\"\n  [1]=>\n  string(3) \"one\"\n  [2]=>\n  string(3) \"two\"\n  [3]=>\n  int(3)\n  [4]=>\n  string(3) \"ten\"\n}\narray(5) {\n  [0]=>\n  string(4) \"zero\"\n  [1]=>\n  string(3) \"one\"\n  [2]=>\n  string(3) \"two\"\n  [\"three\"]=>\n  int(3)\n  [10]=>\n  string(3) \"ten\"\n}\narray(0) {\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_predicates_use_generated_fast_paths() {
    let root = temp_dir("ptn-native-array-predicate-fast-paths");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-predicate-fast-paths.php");
    let output = root.join("array-predicate-fast-paths-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"present\" => 1, \"nullish\" => null, \"zero\" => \"0\"];\n\
var_dump(count($items));\n\
var_dump(array_key_exists(\"nullish\", $items));\n\
var_dump(isset($items[\"present\"]));\n\
var_dump(isset($items[\"nullish\"]));\n\
var_dump(empty($items[\"missing\"]));\n\
var_dump(empty($items[\"zero\"]));\n\
var_dump(empty($missing));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(3)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert!(main_body.contains("ptn_count_value(&runtime"));
    assert!(main_body.contains("ptn_array_key_exists_value(&runtime"));
    assert!(main_body.contains("ptn_offset_is_set(&runtime"));
    assert!(main_body.contains("ptn_offset_is_empty(&runtime"));
    assert!(main_body.contains("ptn_runtime_variable_is_empty(&runtime"));
    assert!(!main_body.contains("ptn_call_function(&runtime, \"count\""));
    assert!(!main_body.contains("ptn_call_function(&runtime, \"array_key_exists\""));
}

#[test]
fn compile_direct_array_helpers_omit_internal_dispatch_block() {
    let root = temp_dir("ptn-native-array-direct-helpers-omit-dispatch");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-direct-helpers-omit-dispatch.php");
    let output = root.join("array-direct-helpers-omit-dispatch-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"present\" => 1, \"missing\" => null];\n\
echo COUNT($items), \"\\n\";\n\
if (ARRAY_KEY_EXISTS(\"present\", $items)) echo \"present\\n\";\n\
if (!array_key_exists(\"absent\", $items)) echo \"absent\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2\npresent\nabsent\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PTN_UNUSED PtnValue ptn_count_value(PtnRuntime *runtime"));
    assert!(c_source.contains("static PTN_UNUSED PtnValue ptn_array_key_exists_value"));
    assert!(c_source.contains("ptn_count_value(&runtime"));
    assert!(c_source.contains("ptn_array_key_exists_value(&runtime"));
    assert!(!c_source.contains("ptn_call_internal"));
    assert!(!c_source.contains("ptn_internal_count"));
    assert!(!c_source.contains("ptn_internal_array_key_exists"));
    assert!(!c_source.contains("ptn_internal_var_dump"));
}

#[test]
fn compile_array_pointer_and_mutation_internals_to_native_binary() {
    let root = temp_dir("ptn-native-array-pointer-mutation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-pointer-mutation.php");
    let output = root.join("array-pointer-mutation-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"a\" => \"apple\", \"b\" => \"book\", \"c\" => \"cook\"];\n\
var_dump(current($items));\n\
var_dump(key($items));\n\
var_dump(next($items));\n\
var_dump(current($items));\n\
var_dump(key($items));\n\
var_dump(end($items));\n\
var_dump(key($items));\n\
var_dump(prev($items));\n\
var_dump(key($items));\n\
var_dump(next($items));\n\
var_dump(next($items));\n\
var_dump(prev($items));\n\
var_dump(key($items));\n\
var_dump(reset($items));\n\
var_dump(key($items));\n\
$empty = [];\n\
var_dump(current($empty));\n\
var_dump(key($empty));\n\
var_dump(end($empty));\n\
var_dump(prev($empty));\n\
var_dump(reset($empty));\n\
$numbers = [1, 2, 3];\n\
var_dump(array_pop($numbers));\n\
var_dump(array_push($numbers, 4, 5));\n\
var_dump($numbers);\n\
$assoc_numbers = [\"3\" => \"foo\", \"4\" => \"bar\", \"5\" => \"fubar\"];\n\
var_dump(array_pop($assoc_numbers));\n\
var_dump($assoc_numbers);\n\
$mixed = [\"x\" => \"ex\", 4 => \"four\", 9 => \"nine\", \"z\" => \"zed\"];\n\
var_dump(array_shift($mixed));\n\
var_dump($mixed);\n\
$copy_source = [[10, 20], [30, 40]];\n\
$copy = $copy_source[0];\n\
var_dump(array_shift($copy));\n\
var_dump($copy_source[0]);\n\
foreach ($copy_source as $sub) { array_shift($sub); }\n\
var_dump($copy_source[1]);\n\
var_dump(function_exists(\"ARRAY_POP\"), function_exists(\"current\"), function_exists(\"end\"), function_exists(\"prev\"), function_exists(\"reset\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"apple\"\nstring(1) \"a\"\nstring(4) \"book\"\nstring(4) \"book\"\nstring(1) \"b\"\nstring(4) \"cook\"\nstring(1) \"c\"\nstring(4) \"book\"\nstring(1) \"b\"\nstring(4) \"cook\"\nbool(false)\nbool(false)\nNULL\nstring(5) \"apple\"\nstring(1) \"a\"\nbool(false)\nNULL\nbool(false)\nbool(false)\nbool(false)\nint(3)\nint(4)\narray(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(4)\n  [3]=>\n  int(5)\n}\nstring(5) \"fubar\"\narray(2) {\n  [3]=>\n  string(3) \"foo\"\n  [4]=>\n  string(3) \"bar\"\n}\nstring(2) \"ex\"\narray(3) {\n  [0]=>\n  string(4) \"four\"\n  [1]=>\n  string(4) \"nine\"\n  [\"z\"]=>\n  string(3) \"zed\"\n}\nint(10)\narray(2) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n}\narray(2) {\n  [0]=>\n  int(30)\n  [1]=>\n  int(40)\n}\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn parser_rejects_temporary_array_cursor_mutation_calls() {
    for (source, function) in [
        ("<?php next([1, 2]);", "next"),
        ("<?php var_dump(reset(array(1, 2)));", "reset"),
        ("<?php $items = [[1], [2]]; end($items[0]);", "end"),
        (
            "<?php $items = [1, 2]; var_dump(prev(current($items)));",
            "prev",
        ),
    ] {
        let error = parser::parse(source).unwrap_err();
        assert!(
            error.message.contains(&format!(
                "{function}() requires a direct variable array argument"
            )),
            "unexpected diagnostic for {function}: {}",
            error.message
        );
        assert!(error
            .message
            .contains("temporary array cursor mutation is unsupported"));
    }

    parser::parse("<?php $items = [1, 2]; next(($items));").unwrap();
}

#[test]
fn compile_large_ordered_array_lookup_to_native_binary() {
    let root = temp_dir("ptn-native-large-array-lookup");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("large-array-lookup.php");
    let output = root.join("large-array-lookup-bin");
    let mut source = String::from("<?php\n$items = [\n");
    for index in 0..64 {
        source.push_str(&format!("\"k{index}\" => {index},\n"));
    }
    source.push_str("\"k10\" => 1000,\n");
    source.push_str("\"20\" => 2000,\n");
    source.push_str("65,\n");
    source.push_str("];\n");
    source.push_str("var_dump($items[\"k10\"]);\n");
    source.push_str("var_dump($items[20]);\n");
    source.push_str("var_dump($items[\"20\"]);\n");
    source.push_str("var_dump(count($items));\n");
    source.push_str("var_dump(array_key_exists(\"20\", $items));\n");
    source.push_str("var_dump(array_key_exists(\"missing\", $items));\n");
    source.push_str("var_dump(isset($items[\"k0\"]));\n");
    source.push_str("var_dump(empty($items[\"missing\"]));\n");
    source.push_str(
        "foreach ($items as $key => $value) { if ($key === \"k10\" || $key === 20 || $key === 21) echo $key, \"=\", $value, \"\\n\"; }\n",
    );
    fs::write(&input, source).unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1000)\nint(2000)\nint(2000)\nint(66)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nk10=1000\n20=2000\n21=65\n"
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
fn compile_numeric_string_offset_reads_to_native_binary() {
    let root = temp_dir("ptn-native-numeric-string-offset-reads");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("numeric-string-offset-reads.php");
    let output = root.join("numeric-string-offset-reads-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"The world is fun\";\n\
$keys = [\n\
    \"7\",\n\
    \"  7\",\n\
    \"  7  \",\n\
    \"7  \",\n\
    \"7str\",\n\
    \"  7str\",\n\
    \"  7  str\",\n\
    \"7  str\",\n\
    \"0xC\",\n\
    \"0b10\",\n\
    \"07\",\n\
];\n\
foreach ($keys as $key) {\n\
    var_dump($str[$key]);\n\
}\n\
echo \"Done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"l\"\nstring(1) \"l\"\nstring(1) \"l\"\nstring(1) \"l\"\n\nWarning: Illegal string offset \"7str\" in ptn on line 17\nstring(1) \"l\"\n\nWarning: Illegal string offset \"  7str\" in ptn on line 17\nstring(1) \"l\"\n\nWarning: Illegal string offset \"  7  str\" in ptn on line 17\nstring(1) \"l\"\n\nWarning: Illegal string offset \"7  str\" in ptn on line 17\nstring(1) \"l\"\n\nWarning: Illegal string offset \"0xC\" in ptn on line 17\nstring(1) \"T\"\n\nWarning: Illegal string offset \"0b10\" in ptn on line 17\nstring(1) \"T\"\nstring(1) \"l\"\nDone\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_offset_writes_to_native_binary() {
    let root = temp_dir("ptn-native-string-offset-writes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-offset-writes.php");
    let output = root.join("string-offset-writes-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"abcd\";\n\
$str[1] = \"XYZ\";\n\
var_dump($str);\n\
$str[-1] = \"Q\";\n\
var_dump($str);\n\
$str[6] = \"Z\";\n\
var_dump($str);\n\
$str[true] = \"T\";\n\
var_dump($str);\n\
$str[null] = \"N\";\n\
var_dump($str);\n\
$str[\"2str\"] = \"R\";\n\
var_dump($str);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nWarning: Only the first byte will be assigned to the string offset in ptn on line 3\nstring(4) \"aXcd\"\nstring(4) \"aXcQ\"\nstring(7) \"aXcQ  Z\"\n\nWarning: String offset cast occurred in ptn on line 9\nstring(7) \"aTcQ  Z\"\n\nWarning: String offset cast occurred in ptn on line 11\nstring(7) \"NTcQ  Z\"\n\nWarning: Illegal string offset \"2str\" in ptn on line 13\nstring(7) \"NTRQ  Z\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_offset_mutation_boundaries_to_native_binary() {
    let root = temp_dir("ptn-native-string-offset-mutation-boundaries");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-offset-mutation-boundaries.php");
    let output = root.join("string-offset-mutation-boundaries-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"abcd\";\n\
try { $str[1] = \"\"; } catch (\\Error $e) { echo $e->getMessage() . PHP_EOL; }\n\
var_dump($str);\n\
try { $str[\"1.5\"] = \"Q\"; } catch (\\TypeError $e) { echo $e->getMessage() . PHP_EOL; }\n\
try { $str[] = \"A\"; } catch (\\Error $e) { echo $e->getMessage() . PHP_EOL; }\n\
try { unset($str[0]); } catch (\\Error $e) { echo $e->getMessage() . PHP_EOL; }\n\
try { $str[1] .= \"Z\"; } catch (\\Error $e) { echo $e->getMessage() . PHP_EOL; }\n\
$str[-5] = \"Z\";\n\
var_dump($str);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot assign an empty string to a string offset\nstring(4) \"abcd\"\nCannot access offset of type string on string\n[] operator not supported for strings\nCannot unset string offsets\nCannot use assign-op operators with string offsets\n\nWarning: Illegal string offset -5 in ptn on line 9\nstring(4) \"abcd\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_try_catches_string_offset_type_error_to_native_binary() {
    let root = temp_dir("ptn-native-try-catch-string-offset-type-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("try-catch-string-offset-type-error.php");
    let output = root.join("try-catch-string-offset-type-error-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"The world is fun\";\n\
try {\n\
    echo $str[\"7.5\"];\n\
    echo \"unreached\\n\";\n\
} catch (\\TypeError $e) {\n\
    echo $e->getMessage() . \\PHP_EOL;\n\
}\n\
echo \"after\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot access offset of type string on string\nafter\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_uncaught_string_offset_type_error_still_fatals() {
    let root = temp_dir("ptn-native-uncaught-string-offset-type-error");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("uncaught-string-offset-type-error.php");
    let output = root.join("uncaught-string-offset-type-error-bin");
    fs::write(
        &input,
        "<?php $str = \"The world is fun\"; echo $str[\"7.5\"]; echo \"unreached\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: Cannot access offset of type string on string\n"
    );
}

#[test]
fn compile_numeric_string_offset_type_errors_inside_foreach_to_native_binary() {
    let root = temp_dir("ptn-native-numeric-string-offset-type-errors");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("numeric-string-offset-type-errors.php");
    let output = root.join("numeric-string-offset-type-errors-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"The world is fun\";\n\
$keys = [\"7\", \"7.5\", \"7str\", \"7.5str\", \"0xC\", \"0b10\", \"07\"];\n\
foreach ($keys as $key) {\n\
    try {\n\
        var_dump($str[$key]);\n\
    } catch (\\TypeError $e) {\n\
        echo $e->getMessage() . \\PHP_EOL;\n\
    }\n\
}\n\
echo \"Done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"l\"\nCannot access offset of type string on string\n\nWarning: Illegal string offset \"7str\" in ptn on line 6\nstring(1) \"l\"\nCannot access offset of type string on string\n\nWarning: Illegal string offset \"0xC\" in ptn on line 6\nstring(1) \"T\"\n\nWarning: Illegal string offset \"0b10\" in ptn on line 6\nstring(1) \"T\"\nstring(1) \"l\"\nDone\n"
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
fn compile_scalar_comparison_fast_paths_preserve_edges_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-comparison-fast-paths");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-comparison-fast-paths.php");
    let output = root.join("scalar-comparison-fast-paths-bin");
    fs::write(
        &input,
        "<?php\n\
$zero = 0;\n\
$zeroFloat = 0.0;\n\
$one = 1;\n\
$twoFloat = 2.5;\n\
$word = \"alpha\";\n\
$same = \"alpha\";\n\
$numeric = \"042\";\n\
$numericFloat = \"42.0\";\n\
echo $zero == $zeroFloat, \"|\", $one < $twoFloat, \"|\", $twoFloat > $one, \"|\", $word === $same, \"|\", $word !== \"beta\", \"|\";\n\
echo $numeric == $numericFloat, \"|\", \"10\" < \"2\", \"|\", \"alpha\" < \"beta\", \"|\", null == \"\", \"|\", null == 0, \"|\", false == \"0\", \"|\", true == \"0\", \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "1|1|1|1|1|1||1|1|1|1|\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PTN_UNUSED int ptn_compare_number_types"));
    assert!(c_source.contains("static PTN_UNUSED int ptn_compare_strings_loose"));
    assert!(c_source.contains("static PTN_UNUSED int ptn_compare_not_identical"));
    assert!(c_source.contains(" = ptn_bool(ptn_compare_not_identical("));
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
        undefined_variable_warning(&input, "missing", 1)
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
fn compile_expression_statements_evaluate_and_discard_to_native_binary() {
    let root = temp_dir("ptn-native-expression-statements");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("expression-statements.php");
    let output = root.join("expression-statements-bin");
    fs::write(
        &input,
        "<?php $value = 1; $value + 2; $missing; strlen(\"abc\"); echo \"done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "done\n");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        undefined_variable_warning(&input, "missing", 1)
    );
}

#[test]
fn compile_switch_return_skips_unreachable_expression_statement_to_native_binary() {
    let root = temp_dir("ptn-native-code-before-loop-var-free");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("code-before-loop-var-free.php");
    let output = root.join("code-before-loop-var-free-bin");
    fs::write(
        &input,
        "<?php
switch ($x > 0) {
default:
    return;
    Y;
}
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        undefined_variable_warning(&input, "x", 2)
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
fn compile_continue_rechecks_while_condition_to_native_binary() {
    let root = temp_dir("ptn-native-while-continue");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("while-continue.php");
    let output = root.join("while-continue-bin");
    fs::write(
        &input,
        "<?php $i = 0; while ($i < 4) { $i++; if ($i == 2) continue; echo $i; } echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "134\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_continue_in_do_while_checks_condition_after_body_to_native_binary() {
    let root = temp_dir("ptn-native-do-while-continue");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("do-while-continue.php");
    let output = root.join("do-while-continue-bin");
    fs::write(
        &input,
        "<?php $i = 0; do { $i++; if ($i == 1) continue; echo $i; } while ($i < 3); echo \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "23\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_for_continue_runs_update_to_native_binary() {
    let root = temp_dir("ptn-native-for-continue-update");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("for-continue-update.php");
    let output = root.join("for-continue-update-bin");
    fs::write(
        &input,
        "<?php for ($i = 0; $i < 5; $i++) { if ($i == 1) continue; if ($i == 3) continue; echo $i; } echo \":$i\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "024:5\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_continue_levels_through_switch_to_native_binary() {
    let root = temp_dir("ptn-native-switch-continue-levels");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("switch-continue-levels.php");
    let output = root.join("switch-continue-levels-bin");
    fs::write(
        &input,
        "<?php $i = 0; while ($i < 2) { switch ($i) { case 0: echo \"case\"; continue; case 1: echo \"one\"; $i++; continue 2; } echo \":after:\"; $i++; } echo \":done:$i\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "\nWarning: \"continue\" targeting switch is equivalent to \"break\". Did you mean to use \"continue 2\"? in {} on line 1\ncase:after:one:done:2\n",
            input.display()
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_unmatched_large_continue_level_reports_source_line_to_native_binary() {
    let root = temp_dir("ptn-native-large-continue-level-fatal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("large-continue-level.php");
    let output = root.join("large-continue-level-bin");
    fs::write(&input, "<?php\nfor(;;) continue 2147483648;\n").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Cannot 'continue' 2147483648 levels in {} on line 2\n",
            input.display()
        )
    );
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
        undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
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
        undefined_variable_warnings(
            &input,
            &[
                ("total", 1),
                ("missing_number", 1),
                ("text", 1),
                ("missing_text", 1),
            ]
        )
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
fn compile_concat_chains_and_compound_loops_to_native_binary() {
    let root = temp_dir("ptn-native-concat-chains-and-loops");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("concat-chains-and-loops.php");
    let output = root.join("concat-chains-and-loops-bin");
    fs::write(
        &input,
        "<?php
$chain = \"\";
$i = 0;
while ($i < 25) {
    $chain = $chain . \"a\" . $i . \"-\";
    $i++;
}
$compound = \"\";
for ($j = 0; $j < 25; $j++) {
    $compound .= \"b\";
    $compound .= $j;
    $compound .= \"|\";
}
echo strlen($chain), \" \", strlen($compound), \"\\n\";
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "90 90\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_concat_chains_emit_single_builder_calls_to_native_binary() {
    let root = temp_dir("ptn-native-concat-chain-builder");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("concat-chain-builder.php");
    let output = root.join("concat-chain-builder-bin");
    fs::write(
        &input,
        "<?php
$a = [1];
$b = [2];
$text = $a . \"x\" . 42 . $b;
$out = \"\";
$i = 7;
$out .= \"b\" . $i . \"|\";
echo $text, \"\\n\", $out, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: Array to string conversion in ptn on line 4\n\
Warning: Array to string conversion in ptn on line 4\n\
Arrayx42Array\n\
b7|\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert_eq!(main_body.matches("ptn_concat_many(&runtime").count(), 2);
    assert!(!main_body.contains("ptn_concat(&runtime"));
    assert!(main_body.contains("PtnConcatOperand"));
}

#[test]
fn compile_owned_temporaries_and_slots_are_destroyed_to_native_binary() {
    let root = temp_dir("ptn-native-owned-temporary-cleanup");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("owned-temporary-cleanup.php");
    let output = root.join("owned-temporary-cleanup-bin");
    fs::write(
        &input,
        "<?php
$value = \"\";
$array = [];
$i = 0;
while ($i < 200) {
    $value = \"prefix-\" . $i . \"-\" . str_rot13(\"abcdefghijk\");
    md5(\"discard-\" . $i);
    $array = [\"payload\" => $value . \"-array\", \"nested\" => [\"n\" => $i . \"-nested\"]];
    $array = [\"replacement\" => $i];
    $i++;
}
echo strlen($value), \" \", count($array), \" \", $array[\"replacement\"], \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "22 1 199\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PTN_UNUSED void ptn_value_destroy(PtnValue *value)"));
    assert!(c_source.contains("ptn_value_destroy(&symbols->items[index].value);"));
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert!(main_body.contains("ptn_value_destroy(&ptn_tmp_"));
}

#[test]
fn compile_array_concat_emits_string_conversion_warnings_to_native_binary() {
    let root = temp_dir("ptn-native-array-concat-warnings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-concat-warnings.php");
    let output = root.join("array-concat-warnings-bin");
    fs::write(
        &input,
        "<?php
$a = [1, 2];
var_dump($a . \"x\");
var_dump(\"x\" . $a);
$a .= [3];
var_dump($a);
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Warning: Array to string conversion in ptn on line 3\n\
string(6) \"Arrayx\"\n\
Warning: Array to string conversion in ptn on line 4\n\
string(6) \"xArray\"\n\
Warning: Array to string conversion in ptn on line 5\n\
Warning: Array to string conversion in ptn on line 5\n\
string(10) \"ArrayArray\"\n"
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
fn compile_common_scalar_numeric_paths_to_native_binary() {
    let root = temp_dir("ptn-native-common-scalar-numeric-paths");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("common-scalar-numeric-paths.php");
    let output = root.join("common-scalar-numeric-paths-bin");
    fs::write(
        &input,
        "<?php $int = 10; $float = 2.5; $truthy = true; $empty = null; var_dump($int + $truthy); var_dump($float * $int); var_dump($int / 2); var_dump($empty + 4); var_dump($int % 4); var_dump($int > 9); var_dump((int)$truthy); var_dump((float)$int); var_dump(fdiv($int, 4));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(11)\nfloat(25)\nint(5)\nint(4)\nint(2)\nbool(true)\nint(1)\nfloat(1E+1)\nfloat(2.5)\n"
    );
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
        undefined_variable_warnings(&input, &[("left", 1), ("right", 1), ("third", 1)])
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

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2 5 5\nstring(3) \"020\"\nstring(8) \"3337>755\"\nstring(4) \"wo\x7fu\"\nstring(6) \"030107\"\nstring(4) \"pead\"\nstring(4) \"wo\x7fu\"\nstring(8) \"070a1e11\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    for marker in [
        "static PTN_UNUSED PtnValue ptn_bitwise_string_and(",
        "static PTN_UNUSED PtnValue ptn_bitwise_string_or(",
        "static PTN_UNUSED PtnValue ptn_bitwise_string_xor(",
    ] {
        let body = generated_c_static_function_body(&c_source, marker);
        assert!(
            !body.contains("strlen("),
            "{marker} should consume caller-provided lengths instead of rescanning"
        );
    }
    assert!(c_source.contains(
        "static PTN_UNUSED PtnValue ptn_bitwise_string_and(PtnString left, PtnString right)"
    ));
    assert!(c_source.contains(
        "static PTN_UNUSED PtnValue ptn_bitwise_string_or(PtnString left, PtnString right)"
    ));
    assert!(c_source.contains(
        "static PTN_UNUSED PtnValue ptn_bitwise_string_xor(PtnString left, PtnString right)"
    ));
    assert!(c_source.contains("size_t left_len = left.len;"));
    assert!(c_source.contains("size_t right_len = right.len;"));
    assert!(c_source.contains("ptn_bitwise_string_and(left.as.string, right.as.string)"));
    assert!(c_source.contains("ptn_bitwise_string_or(left.as.string, right.as.string)"));
    assert!(c_source.contains("ptn_bitwise_string_xor(left.as.string, right.as.string)"));
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

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(-24)\nstring(8) \"8c90929a\"\n\nDeprecated: Implicit conversion from float 23.67 to int loses precision in ptn-generated-code on line 0\nint(-24)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let body = generated_c_static_function_body(
        &c_source,
        "static PTN_UNUSED PtnValue ptn_bitwise_string_not(",
    );
    assert!(
        !body.contains("strlen("),
        "ptn_bitwise_string_not should consume caller-provided lengths instead of rescanning"
    );
    assert!(body.contains("size_t len = value.len;"));
    assert!(c_source.contains("ptn_bitwise_string_not(value.as.string)"));
}

#[test]
fn compile_unary_bitwise_not_array_operand_fatals() {
    let root = temp_dir("ptn-native-bitwise-not-array");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-not-array.php");
    let output = root.join("bitwise-not-array-bin");
    fs::write(&input, "<?php\n$value = [1, 2];\nvar_dump(~$value);").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: Cannot perform bitwise not on array in {} on line 3\n",
            input.display()
        )
    );
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
        undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
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
        undefined_variable_warning(&input, "missing", 1)
    );
}

#[test]
fn compile_many_runtime_symbols_to_native_binary() {
    let root = temp_dir("ptn-native-many-runtime-symbols");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("many-runtime-symbols.php");
    let output = root.join("many-runtime-symbols-bin");

    let mut source = String::from("<?php\n");
    for i in 0..128 {
        source.push_str(&format!("$v{i} = {i};\n"));
    }
    source.push_str("$v70 = 7000;\n");
    for i in 0..128 {
        source.push_str(&format!("define(\"C{i}\", {i});\n"));
    }
    source.push_str("$sum = 0;\n");
    source.push_str("for ($i = 0; $i < 8; $i++) {\n");
    source.push_str("    $sum += $v0 + $v15 + $v70 + $v127;\n");
    source.push_str("    $sum += constant(\"C0\") + constant(\"C15\") + constant(\"C70\") + constant(\"C127\");\n");
    source.push_str("}\n");
    source.push_str(
        "var_dump($sum, defined(\"C127\"), constant(\"C127\"), isset($v127), isset($missing));\n",
    );
    fs::write(&input, source).unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(58832)\nbool(true)\nint(127)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn unsupported_constructs_fail_before_codegen() {
    let unsupported_operator = parser::parse("<?php $name ??= 1;").unwrap_err();
    assert!(unsupported_operator.message.contains("expected expression"));
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
    assert!(support.contains("recursive arrays"));
    assert!(support.contains("objects"));
    assert!(support.contains("resources"));
    assert!(support.contains("references"));
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
