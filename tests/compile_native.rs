use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{
    ArrayElementValue, AssignmentOp, AssignmentTarget, BinaryOp, CastKind, Expr, IncDecOp,
    IncDecResult, IncDecTarget, IncludeKind, ListAssignmentElementTarget, MagicConstantKind,
    PropertyVisibility, ReferenceTarget, Statement, StringInterpolationIndex, StringPart, TypeHint,
    UnaryOp, UnsetTarget,
};
use ptn::lexer::{self, TokenKind};
use ptn::{compile_file, parser, CompileOptions, DiagnosticKind};

fn undefined_variable_warning(path: &Path, name: &str, line: usize) -> String {
    format!(
        "\nWarning: Undefined variable ${name} in {} on line {line}\n",
        path.display()
    )
}

fn undefined_variable_warnings(path: &Path, warnings: &[(&str, usize)]) -> String {
    let mut output = String::new();
    for (name, line) in warnings {
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
fn lexer_decodes_ascii_double_quoted_octal_and_hex_escapes() {
    let tokens = lexer::lex("<?php \"a\\145..\\160z\" \"\\x41\\x7a\" \"\\0\"").unwrap();
    assert!(matches!(&tokens[1].kind, TokenKind::String(value) if value == "ae..pz"));
    assert!(matches!(&tokens[2].kind, TokenKind::String(value) if value == "Az"));
    assert!(matches!(&tokens[3].kind, TokenKind::String(value) if value.as_bytes() == [0]));
}

#[test]
fn lexer_accepts_plain_heredoc_and_nowdoc_strings() {
    let source = "<?php $left = <<<TXT\nHello\nTXT;\n$right = <<<'TXT'\n$literal\nTXT;\n";
    let program = parser::parse(source).unwrap();
    let Statement::Assign { value, .. } = &program.statements[0] else {
        panic!("expected assignment");
    };
    assert!(matches!(value, Expr::String(value, _) if value == "Hello"));

    let Statement::Assign { value, .. } = &program.statements[1] else {
        panic!("expected assignment");
    };
    assert!(matches!(value, Expr::String(value, _) if value == "$literal"));
}

#[test]
fn lexer_rejects_interpolating_heredoc_bodies() {
    let error = lexer::lex("<?php $value = <<<TXT\nHello $name\nTXT;\n").unwrap_err();
    assert_eq!(error.message, "heredoc interpolation is unsupported");
    assert_eq!(error.span.unwrap().line, 2);
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
        "<?php $value = 1; $value ??= 13; $value += 2; $value -= 3; $value *= 4; $value **= 2; $value /= 5; $value %= 6; $value .= \"7\"; $value &= \"8\"; $value |= \"9\"; $value ^= \"10\"; $value <<= 11; $value >>= 12;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 14);

    let Statement::Assign { op, .. } = &program.statements[1] else {
        panic!("expected null coalescing assignment statement");
    };
    assert_eq!(*op, AssignmentOp::CoalesceAssign);

    let Statement::Assign { op, .. } = &program.statements[2] else {
        panic!("expected add assignment statement");
    };
    assert_eq!(*op, AssignmentOp::AddAssign);

    let Statement::Assign { op, .. } = &program.statements[3] else {
        panic!("expected subtract assignment statement");
    };
    assert_eq!(*op, AssignmentOp::SubtractAssign);

    let Statement::Assign { op, .. } = &program.statements[4] else {
        panic!("expected multiply assignment statement");
    };
    assert_eq!(*op, AssignmentOp::MultiplyAssign);

    let Statement::Assign { op, .. } = &program.statements[5] else {
        panic!("expected power assignment statement");
    };
    assert_eq!(*op, AssignmentOp::PowerAssign);

    let Statement::Assign { op, .. } = &program.statements[6] else {
        panic!("expected divide assignment statement");
    };
    assert_eq!(*op, AssignmentOp::DivideAssign);

    let Statement::Assign { op, .. } = &program.statements[7] else {
        panic!("expected modulo assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ModuloAssign);

    let Statement::Assign { op, .. } = &program.statements[8] else {
        panic!("expected concat assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ConcatAssign);

    let Statement::Assign { op, .. } = &program.statements[9] else {
        panic!("expected bitwise and assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseAndAssign);

    let Statement::Assign { op, .. } = &program.statements[10] else {
        panic!("expected bitwise or assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseOrAssign);

    let Statement::Assign { op, .. } = &program.statements[11] else {
        panic!("expected bitwise xor assignment statement");
    };
    assert_eq!(*op, AssignmentOp::BitwiseXorAssign);

    let Statement::Assign { op, .. } = &program.statements[12] else {
        panic!("expected shift left assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ShiftLeftAssign);

    let Statement::Assign { op, .. } = &program.statements[13] else {
        panic!("expected shift right assignment statement");
    };
    assert_eq!(*op, AssignmentOp::ShiftRightAssign);
}

#[test]
fn parser_accepts_assignment_expressions_in_branch_conditions() {
    let program = parser::parse(
        "<?php if ($value += 1) { echo $value; } while ($value .= \"x\") { break; } for (; $value ??= \"fallback\"; ) { break; }",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 3);

    let Statement::If { condition, .. } = &program.statements[0] else {
        panic!("expected if statement");
    };
    let Expr::Assign { op, .. } = condition else {
        panic!("expected assignment expression in if condition");
    };
    assert_eq!(*op, AssignmentOp::AddAssign);

    let Statement::While { condition, .. } = &program.statements[1] else {
        panic!("expected while statement");
    };
    let Expr::Assign { op, .. } = condition else {
        panic!("expected assignment expression in while condition");
    };
    assert_eq!(*op, AssignmentOp::ConcatAssign);

    let Statement::For {
        condition: Some(condition),
        ..
    } = &program.statements[2]
    else {
        panic!("expected for statement condition");
    };
    let Expr::Assign { op, .. } = condition else {
        panic!("expected assignment expression in for condition");
    };
    assert_eq!(*op, AssignmentOp::CoalesceAssign);
}

#[test]
fn parser_accepts_direct_variable_increment_decrement_statements() {
    let program = parser::parse("<?php $value = 1; $value++; ++$value; $value--; --$value; $items[0]++; --$items[$key]; while ($value < 3) { $value++; }").unwrap();
    assert_eq!(program.statements.len(), 8);

    let Statement::Increment { op, .. } = &program.statements[1] else {
        panic!("expected postfix increment statement");
    };
    assert_eq!(*op, IncDecOp::Increment);

    let Statement::Increment { op, .. } = &program.statements[4] else {
        panic!("expected prefix decrement statement");
    };
    assert_eq!(*op, IncDecOp::Decrement);

    assert!(matches!(
        &program.statements[5],
        Statement::Increment {
            target: IncDecTarget::ArrayDim(target),
            op: IncDecOp::Increment,
            ..
        } if target.array == "items" && target.dimensions.len() == 1
    ));
    assert!(matches!(
        &program.statements[6],
        Statement::Increment {
            target: IncDecTarget::ArrayDim(target),
            op: IncDecOp::Decrement,
            ..
        } if target.array == "items" && target.dimensions.len() == 1
    ));

    let Statement::While { body, .. } = &program.statements[7] else {
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
        value_by_ref,
        body,
        ..
    } = &program.statements[0]
    else {
        panic!("expected value-only foreach statement");
    };
    assert!(matches!(iterable, Expr::Variable(name, _) if name == "items"));
    assert_eq!(key, &None);
    assert!(matches!(
        value,
        AssignmentTarget::Variable { name, .. } if name == "value"
    ));
    assert!(!value_by_ref);
    assert_eq!(body.len(), 1);

    let Statement::Foreach {
        key,
        value,
        value_by_ref,
        body,
        ..
    } = &program.statements[1]
    else {
        panic!("expected key/value foreach statement");
    };
    assert!(matches!(
        key,
        Some(AssignmentTarget::Variable { name, .. }) if name == "key"
    ));
    assert!(matches!(
        value,
        AssignmentTarget::Variable { name, .. } if name == "value"
    ));
    assert!(!value_by_ref);
    assert_eq!(body.len(), 1);
}

#[test]
fn parser_accepts_foreach_array_dim_binding_targets() {
    let program =
        parser::parse("<?php foreach ($items as $keys[0] => $values[$i]) { echo $i; }").unwrap();

    let Statement::Foreach { key, value, .. } = &program.statements[0] else {
        panic!("expected foreach statement");
    };
    let Some(AssignmentTarget::ArrayDim(key_target)) = key else {
        panic!("expected array-dimension key target");
    };
    assert_eq!(key_target.array, "keys");
    assert_eq!(key_target.dimensions.len(), 1);
    let AssignmentTarget::ArrayDim(value_target) = value else {
        panic!("expected array-dimension value target");
    };
    assert_eq!(value_target.array, "values");
    assert_eq!(value_target.dimensions.len(), 1);
}

#[test]
fn parser_accepts_empty_statements_as_loop_bodies() {
    let program = parser::parse("<?php ; foreach ($items as $value); while ($ready);").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(&program.statements[0], Statement::Empty { .. }));

    let Statement::Foreach { body, .. } = &program.statements[1] else {
        panic!("expected foreach statement");
    };
    assert_eq!(body.len(), 1);
    assert!(matches!(&body[0], Statement::Empty { .. }));

    let Statement::While { body, .. } = &program.statements[2] else {
        panic!("expected while statement");
    };
    assert_eq!(body.len(), 1);
    assert!(matches!(&body[0], Statement::Empty { .. }));
}

#[test]
fn parser_accepts_by_reference_foreach_value_binding() {
    let program = parser::parse(
        "<?php foreach ($items as &$value) { echo $value; } foreach ($items as $key => &$value) echo $key;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Foreach {
        key,
        value,
        value_by_ref,
        ..
    } = &program.statements[0]
    else {
        panic!("expected value-only foreach statement");
    };
    assert_eq!(key, &None);
    assert!(matches!(
        value,
        AssignmentTarget::Variable { name, .. } if name == "value"
    ));
    assert!(*value_by_ref);

    let Statement::Foreach {
        key,
        value,
        value_by_ref,
        ..
    } = &program.statements[1]
    else {
        panic!("expected key/value foreach statement");
    };
    assert!(matches!(
        key,
        Some(AssignmentTarget::Variable { name, .. }) if name == "key"
    ));
    assert!(matches!(
        value,
        AssignmentTarget::Variable { name, .. } if name == "value"
    ));
    assert!(*value_by_ref);
}

#[test]
fn parser_accepts_foreach_list_destructuring_bindings() {
    let program = parser::parse(
        "<?php foreach ($items as [$value, \"name\" => $named, [&$ref]]) { echo $value; }",
    )
    .unwrap();

    let Statement::Foreach {
        key,
        value,
        value_by_ref,
        ..
    } = &program.statements[0]
    else {
        panic!("expected foreach statement");
    };
    assert_eq!(key, &None);
    assert!(!*value_by_ref);

    let AssignmentTarget::List(target) = value else {
        panic!("expected foreach destructuring target");
    };
    assert_eq!(target.elements.len(), 3);
    assert!(target.elements[0].key.is_none());
    assert!(matches!(
        &target.elements[0].target,
        ListAssignmentElementTarget::Value(target)
            if matches!(target.as_ref(), AssignmentTarget::Variable { name, .. } if name == "value")
    ));
    assert!(matches!(
        &target.elements[1].target,
        ListAssignmentElementTarget::Value(target)
            if matches!(target.as_ref(), AssignmentTarget::Variable { name, .. } if name == "named")
    ));
    let ListAssignmentElementTarget::Value(nested) = &target.elements[2].target else {
        panic!("expected nested list target");
    };
    let AssignmentTarget::List(nested) = nested.as_ref() else {
        panic!("expected nested list target");
    };
    assert!(matches!(
        &nested.elements[0].target,
        ListAssignmentElementTarget::Reference(ReferenceTarget::Variable { name, .. })
            if name == "ref"
    ));
}

#[test]
fn parser_rejects_unsupported_foreach_bindings() {
    let by_ref_key =
        parser::parse("<?php foreach ($items as &$key => $value) { echo $value; }").unwrap_err();
    assert_eq!(by_ref_key.message, "Key element cannot be a reference");

    let destructuring =
        parser::parse("<?php foreach ($items as &[$value]) { echo $value; }").unwrap_err();
    assert_eq!(
        destructuring.message,
        "foreach destructuring is unsupported"
    );
}

#[test]
fn parser_accepts_print_expression_contexts() {
    let program = parser::parse(
        "<?php $result = print \"hello\"; echo print \"x\"; $sum = 2 + print \"y\"; $paren = print(\"z\");",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 4);

    let Statement::Assign { value, .. } = &program.statements[0] else {
        panic!("expected assignment from print expression");
    };
    assert!(matches!(value, Expr::Print { .. }));

    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected echo with print operand");
    };
    assert!(matches!(&expressions[0], Expr::Print { .. }));

    let Statement::Assign { value, .. } = &program.statements[2] else {
        panic!("expected assignment from binary expression");
    };
    assert!(matches!(
        value,
        Expr::Binary {
            op: BinaryOp::Add,
            right,
            ..
        } if matches!(right.as_ref(), Expr::Print { .. })
    ));

    let Statement::Assign { value, .. } = &program.statements[3] else {
        panic!("expected parenthesized print assignment");
    };
    assert!(matches!(value, Expr::Print { .. }));
}

#[test]
fn parser_accepts_include_expression_contexts() {
    let program = parser::parse(
        "<?php $result = include \"value.php\"; echo require(__DIR__ . \"/plain.php\"); $once = include_once \"once.php\"; echo require_once(__DIR__ . \"/required.php\");",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 4);

    let Statement::Assign { value, .. } = &program.statements[0] else {
        panic!("expected assignment from include expression");
    };
    assert!(matches!(
        value,
        Expr::Include {
            kind: IncludeKind::Include,
            ..
        }
    ));

    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected echo with require operand");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Include {
            kind: IncludeKind::Require,
            ..
        }
    ));

    let Statement::Assign { value, .. } = &program.statements[2] else {
        panic!("expected assignment from include_once expression");
    };
    assert!(matches!(
        value,
        Expr::Include {
            kind: IncludeKind::IncludeOnce,
            ..
        }
    ));

    let Statement::Echo { expressions, .. } = &program.statements[3] else {
        panic!("expected echo with require_once operand");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Include {
            kind: IncludeKind::RequireOnce,
            ..
        }
    ));
}

#[test]
fn parser_accepts_direct_variable_increment_and_decrement_expression_contexts() {
    let program =
        parser::parse("<?php echo ++$value, $value--; $after = --$value + $value++;").unwrap();

    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::IncDec {
            target: IncDecTarget::Variable { name, .. },
            op: IncDecOp::Increment,
            result: IncDecResult::Pre,
            ..
        } if name == "value"
    ));
    assert!(matches!(
        &expressions[1],
        Expr::IncDec {
            target: IncDecTarget::Variable { name, .. },
            op: IncDecOp::Decrement,
            result: IncDecResult::Post,
            ..
        } if name == "value"
    ));

    let Statement::Assign { value, .. } = &program.statements[1] else {
        panic!("expected assignment statement");
    };
    assert!(matches!(
        value,
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
            ..
        } if matches!(
            left.as_ref(),
            Expr::IncDec {
                op: IncDecOp::Decrement,
                result: IncDecResult::Pre,
                ..
            }
        ) && matches!(
            right.as_ref(),
            Expr::IncDec {
                op: IncDecOp::Increment,
                result: IncDecResult::Post,
                ..
            }
        )
    ));
}

#[test]
fn parser_accepts_array_offset_increment_and_decrement_expression_targets() {
    let program = parser::parse("<?php echo ++$items[$key], $items[0]--; $items[$key]++;").unwrap();

    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::IncDec {
            target: IncDecTarget::ArrayDim(target),
            op: IncDecOp::Increment,
            result: IncDecResult::Pre,
            ..
        } if target.array == "items" && target.dimensions.len() == 1
    ));
    assert!(matches!(
        &expressions[1],
        Expr::IncDec {
            target: IncDecTarget::ArrayDim(target),
            op: IncDecOp::Decrement,
            result: IncDecResult::Post,
            ..
        } if target.array == "items" && target.dimensions.len() == 1
    ));

    assert!(matches!(
        &program.statements[1],
        Statement::Increment {
            target: IncDecTarget::ArrayDim(target),
            op: IncDecOp::Increment,
            ..
        } if target.array == "items" && target.dimensions.len() == 1
    ));
}

#[test]
fn parser_accepts_dynamic_increment_and_decrement_expression_targets() {
    let program = parser::parse("<?php echo ++$$name, ${$name}[$key]--;").unwrap();

    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::IncDec {
            target: IncDecTarget::DynamicVariable { name, .. },
            op: IncDecOp::Increment,
            result: IncDecResult::Pre,
            ..
        } if matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
    ));
    assert!(matches!(
        &expressions[1],
        Expr::IncDec {
            target:
                IncDecTarget::DynamicArrayDim {
                    name,
                    dimensions,
                    ..
                },
            op: IncDecOp::Decrement,
            result: IncDecResult::Post,
            ..
        } if matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
            && dimensions.len() == 1
    ));
}

#[test]
fn parser_rejects_invalid_increment_and_decrement_expression_targets() {
    let invalid_prefix = parser::parse("<?php echo ++1;").unwrap_err();
    assert!(invalid_prefix
        .message
        .contains("increment/decrement expression target must be a variable"));

    let invalid_postfix = parser::parse("<?php echo [1][0]++;").unwrap_err();
    assert!(invalid_postfix.message.contains(
        "increment/decrement expression target must be a variable, array offset, or property"
    ));
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
fn parser_accepts_ternary_expressions() {
    let program = parser::parse("<?php echo $a > $b ? 1 : -1, $name ?: \"fallback\";").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert_eq!(expressions.len(), 2);

    let Expr::Ternary {
        condition,
        if_true,
        if_false,
        ..
    } = &expressions[0]
    else {
        panic!("expected full ternary expression");
    };
    assert!(matches!(
        condition.as_ref(),
        Expr::Binary {
            op: BinaryOp::Greater,
            ..
        }
    ));
    assert!(matches!(if_true.as_deref(), Some(Expr::Int(1, _))));
    assert!(matches!(
        if_false.as_ref(),
        Expr::Unary {
            op: UnaryOp::Negate,
            ..
        }
    ));

    let Expr::Ternary {
        if_true, if_false, ..
    } = &expressions[1]
    else {
        panic!("expected short ternary expression");
    };
    assert!(if_true.is_none());
    assert!(matches!(if_false.as_ref(), Expr::String(value, _) if value == "fallback"));
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
fn parser_accepts_inline_html_after_close_tag_as_output() {
    let program = parser::parse("<?php print \"ok\"; ?> html").unwrap();
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(
        &program.statements[1],
        Statement::InlineHtml { content, .. } if content == " html"
    ));
}

#[test]
fn parser_accepts_inline_html_before_open_tag_as_output() {
    let program = parser::parse("#<?php echo 1; ?>").unwrap();
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(
        &program.statements[0],
        Statement::InlineHtml { content, .. } if content == "#"
    ));
    assert!(matches!(
        &program.statements[1],
        Statement::Echo { expressions, .. } if expressions.len() == 1
    ));
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
fn parser_accepts_inline_html_between_php_blocks_as_output() {
    let program = parser::parse("<?php print \"a\"; ?><b><?php echo \"c\"; ?>d").unwrap();
    assert_eq!(program.statements.len(), 4);
    assert!(matches!(&program.statements[0], Statement::Print { .. }));
    assert!(matches!(
        &program.statements[1],
        Statement::InlineHtml { content, .. } if content == "<b>"
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::Echo { expressions, .. } if expressions.len() == 1
    ));
    assert!(matches!(
        &program.statements[3],
        Statement::InlineHtml { content, .. } if content == "d"
    ));
}

#[test]
fn compile_inline_html_between_php_blocks_to_native_binary() {
    let root = temp_dir("ptn-native-inline-html-between-blocks");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inline-html-between-blocks.php");
    let output = root.join("inline-html-between-blocks-bin");
    fs::write(
        &input,
        "#<?php echo 1; ?>\n<?php if (1) { ?>#<?php } ?>\n#<?php echo 1; ?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "#1##1");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn parser_accepts_dynamic_function_value_calls() {
    let program =
        parser::parse("<?php $fn = \"strlen\"; echo $fn(\"abc\"); $fn(\"ignored\");").unwrap();
    assert_eq!(program.statements.len(), 3);
    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::DynamicCall { callee, arguments, .. }
            if matches!(callee.as_ref(), Expr::Variable(name, _) if name == "fn")
                && arguments.len() == 1
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::Expression {
            expression: Expr::DynamicCall { arguments, .. },
            ..
        } if arguments.len() == 1
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
fn parser_accepts_scalar_function_parameter_defaults() {
    let program = parser::parse(
        "<?php function defaults($a = 1, $b = \"two\", $c = false, $d = null, $e = [\"nested\" => [1]]) { return $a; }",
    )
    .unwrap();

    let function = &program.functions[0];
    assert_eq!(function.parameters.len(), 5);
    assert!(matches!(
        function.parameters[0].default_value,
        Some(Expr::Int(1, _))
    ));
    assert!(matches!(
        function.parameters[1].default_value,
        Some(Expr::String(ref value, _)) if value == "two"
    ));
    assert!(matches!(
        function.parameters[2].default_value,
        Some(Expr::Bool(false, _))
    ));
    assert!(matches!(
        function.parameters[3].default_value,
        Some(Expr::Null(_))
    ));
    assert!(matches!(
        function.parameters[4].default_value,
        Some(Expr::Array { .. })
    ));
}

#[test]
fn parser_accepts_array_function_parameter_defaults() {
    let program = parser::parse(
        "<?php function defaults($a = [1, \"key\" => [false]], $b = array()) { return $a; }",
    )
    .unwrap();

    let function = &program.functions[0];
    assert_eq!(function.parameters.len(), 2);
    assert!(matches!(
        function.parameters[0].default_value,
        Some(Expr::Array { ref elements, .. }) if elements.len() == 2
    ));
    assert!(matches!(
        function.parameters[1].default_value,
        Some(Expr::Array { ref elements, .. }) if elements.is_empty()
    ));
}

#[test]
fn parser_rejects_unsupported_function_parameter_default_expression() {
    let error = parser::parse("<?php function unsupported($value = strlen('x')) {}").unwrap_err();
    assert_eq!(
        error.message,
        "function parameter default value must be a supported constant expression"
    );
}

#[test]
fn parser_rejects_required_parameter_after_optional_parameter() {
    let error =
        parser::parse("<?php function unsupported($optional = 1, $required) {}").unwrap_err();
    assert_eq!(
        error.message,
        "required function parameter cannot follow an optional parameter"
    );
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
fn parser_accepts_scalar_parameter_return_hints_and_by_ref_returns() {
    let program = parser::parse(
        "<?php function &test(int $a, string &$b): string { return $b; } var_dump(test(1, $x));",
    )
    .unwrap();

    let function = &program.functions[0];
    assert!(function.return_by_ref);
    assert_eq!(function.return_type, Some(TypeHint::String));
    assert_eq!(function.parameters[0].type_hint, Some(TypeHint::Int));
    assert!(!function.parameters[0].by_ref);
    assert_eq!(function.parameters[1].type_hint, Some(TypeHint::String));
    assert!(function.parameters[1].by_ref);
}

#[test]
fn parser_accepts_void_return_type_but_not_void_parameters() {
    let program = parser::parse("<?php function test(): void { return; } test();").unwrap();
    assert_eq!(program.functions[0].return_type, Some(TypeHint::Void));

    let error = parser::parse("<?php function test(void $value) {}").unwrap_err();
    assert_eq!(error.message, "expected function parameter variable");
}

#[test]
fn parser_rejects_value_return_from_void_function() {
    let error = parser::parse("<?php function test(): void { return 1; }").unwrap_err();
    assert_eq!(error.message, "A void function must not return a value");
}

#[test]
fn parser_accepts_variadic_function_parameters() {
    let program =
        parser::parse("<?php function test(int $head, string &...$tail) { return $head; }")
            .unwrap();

    let function = &program.functions[0];
    assert_eq!(function.parameters.len(), 2);
    assert!(!function.parameters[0].is_variadic);
    assert!(function.parameters[1].is_variadic);
    assert!(function.parameters[1].by_ref);
    assert_eq!(function.parameters[1].type_hint, Some(TypeHint::String));

    let error = parser::parse("<?php function invalid(...$head, $tail) {}").unwrap_err();
    assert_eq!(error.message, "Only the last parameter can be variadic");
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

    let error =
        parser::parse("<?php function Explode($separator, $string) { return []; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function explode()");

    let error =
        parser::parse("<?php function STR_REPEAT($value, $times) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_repeat()");

    let error =
        parser::parse("<?php function STR_SPLIT($value, $length = 1) { return []; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_split()");

    let error = parser::parse("<?php function Strip_Tags($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strip_tags()");

    let error =
        parser::parse("<?php function STR_STARTS_WITH($haystack, $needle) { return true; }")
            .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_starts_with()");

    let error = parser::parse("<?php function Str_Ends_With($haystack, $needle) { return true; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function str_ends_with()");

    let error =
        parser::parse("<?php function StrCaseCmp($left, $right) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strcasecmp()");

    let error =
        parser::parse("<?php function StrNCmp($left, $right, $length) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strncmp()");

    let error = parser::parse("<?php function Crc32($value) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function crc32()");

    let error = parser::parse("<?php function STRTOLOWER($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strtolower()");

    let error = parser::parse("<?php function StrToUpper($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strtoupper()");

    let error = parser::parse("<?php function Trim($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function trim()");

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

    let error = parser::parse("<?php function Zend_Version() { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function zend_version()");

    let error = parser::parse("<?php function Get_Loaded_Extensions() { return []; }").unwrap_err();
    assert_eq!(
        error.message,
        "Cannot redeclare function get_loaded_extensions()"
    );

    let error = parser::parse("<?php function IsSet($value) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function isset()");

    let error = parser::parse("<?php function Empty($value) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function empty()");

    let error = parser::parse("<?php function Abs($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function abs()");

    let error = parser::parse("<?php function Count($value) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function count()");

    let error = parser::parse("<?php function SizeOf($value) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function sizeof()");

    let error = parser::parse("<?php function IntDiv($a, $b) { return $a; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function intdiv()");

    let error = parser::parse("<?php function ARRAY_KEY_EXISTS($key, $array) { return null; }")
        .unwrap_err();
    assert_eq!(
        error.message,
        "Cannot redeclare function array_key_exists()"
    );

    let error = parser::parse("<?php function Array_Keys($array) { return []; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_keys()");

    let error = parser::parse("<?php function ARRAY_SEARCH($needle, $haystack) { return false; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_search()");

    let error =
        parser::parse("<?php function Array_Key_First($array) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_key_first()");

    let error =
        parser::parse("<?php function Array_Key_Last($array) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_key_last()");

    let error = parser::parse("<?php function NatSort($array) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function natsort()");

    let error = parser::parse("<?php function StrRev($string) { return $string; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function strrev()");

    let error = parser::parse("<?php function UcFirst($string) { return $string; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function ucfirst()");

    let error = parser::parse("<?php function LcFirst($string) { return $string; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function lcfirst()");

    let error =
        parser::parse("<?php function array_combine($keys, $values) { return []; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_combine()");

    let error =
        parser::parse("<?php function ARRAY_FILTER($array, $callback = null) { return []; }")
            .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_filter()");

    let error = parser::parse("<?php function array_is_list($array) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_is_list()");

    let error =
        parser::parse("<?php function ARRAY_MAP($callback, $array) { return []; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_map()");

    let error = parser::parse("<?php function array_reduce($array, $callback) { return null; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_reduce()");

    let error =
        parser::parse("<?php function array_walk($array, $callback) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_walk()");

    let error =
        parser::parse("<?php function CALL_USER_FUNC($callback) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function call_user_func()");

    let error =
        parser::parse("<?php function call_user_func_array($callback, $args) { return null; }")
            .unwrap_err();
    assert_eq!(
        error.message,
        "Cannot redeclare function call_user_func_array()"
    );

    let error = parser::parse("<?php function Class_Exists($class) { return false; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function class_exists()");

    let error = parser::parse("<?php function method_exists($object, $method) { return false; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function method_exists()");

    let error =
        parser::parse("<?php function Property_Exists($object, $property) { return false; }")
            .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function property_exists()");

    let error = parser::parse("<?php function is_callable($value) { return false; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function is_callable()");

    let error = parser::parse("<?php function Assert($value) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function assert()");

    let error =
        parser::parse("<?php function debug_zval_dump($value) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function debug_zval_dump()");

    let error = parser::parse("<?php function End($array) { return $array; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function end()");

    let error = parser::parse("<?php function Prev($array) { return $array; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function prev()");

    let error = parser::parse("<?php function Print_R($value) { return $value; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function print_r()");

    let error = parser::parse("<?php function JSON_ENCODE($value) { return ''; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function json_encode()");

    let error = parser::parse("<?php function Printf($format) { return 0; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function printf()");

    let error = parser::parse("<?php function array_values($array) { return null; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_values()");

    let error = parser::parse("<?php function is_countable($value) { return false; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function is_countable()");

    let error =
        parser::parse("<?php function IN_ARRAY($needle, $haystack) { return true; }").unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function in_array()");

    let error = parser::parse("<?php function array_search($needle, $haystack) { return false; }")
        .unwrap_err();
    assert_eq!(error.message, "Cannot redeclare function array_search()");
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
fn parser_accepts_keyword_boolean_tail_after_direct_assignment_statement() {
    let program = parser::parse("<?php $result = true and false;").unwrap();
    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected expression statement");
    };
    let Expr::Binary {
        op: BinaryOp::And,
        left,
        ..
    } = expression
    else {
        panic!("expected keyword boolean tail");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Assign {
            op: AssignmentOp::Assign,
            ..
        }
    ));

    let compound = parser::parse("<?php $result += true xor false;").unwrap();
    let Statement::Expression { expression, .. } = &compound.statements[0] else {
        panic!("expected expression statement");
    };
    let Expr::Binary {
        op: BinaryOp::Xor,
        left,
        ..
    } = expression
    else {
        panic!("expected keyword boolean tail");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Assign {
            op: AssignmentOp::AddAssign,
            ..
        }
    ));

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
    assert!(matches!(
        &elements[2].value,
        ArrayElementValue::Value(Expr::Array { .. })
    ));
    assert!(matches!(right.as_ref(), Expr::Array { elements, .. } if elements.is_empty()));
}

#[test]
fn parser_accepts_array_literal_reference_elements() {
    let program = parser::parse("<?php $array = [&$value, \"k\" => &$items[0]];").unwrap();
    let Statement::Assign { value, .. } = &program.statements[0] else {
        panic!("expected array assignment");
    };
    let Expr::Array { elements, .. } = value else {
        panic!("expected array literal");
    };
    assert_eq!(elements.len(), 2);
    assert!(matches!(
        &elements[0].value,
        ArrayElementValue::Reference(ReferenceTarget::Variable { name, .. }) if name == "value"
    ));
    assert!(matches!(
        &elements[1].value,
        ArrayElementValue::Reference(ReferenceTarget::ArrayDim(target))
            if target.array == "items" && target.dimensions.len() == 1
    ));
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
            && matches!(index.as_deref(), Some(Expr::String(value, _)) if value == "7")
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
fn parser_accepts_append_and_list_assignment_expressions() {
    let program = parser::parse("<?php var_dump($ary[] = [&$x] = $x);").unwrap();
    let Statement::Call {
        name, arguments, ..
    } = &program.statements[0]
    else {
        panic!("expected var_dump call");
    };
    assert_eq!(name, "var_dump");
    let Expr::Assign {
        target,
        value: inner,
        ..
    } = &arguments[0]
    else {
        panic!("expected outer assignment expression");
    };
    let AssignmentTarget::ArrayDim(target) = target else {
        panic!("expected append array assignment target");
    };
    assert_eq!(target.array, "ary");
    assert_eq!(target.dimensions, vec![None]);

    let Expr::Assign { target, value, .. } = inner.as_ref() else {
        panic!("expected inner list assignment expression");
    };
    let AssignmentTarget::List(target) = target else {
        panic!("expected list assignment target");
    };
    assert_eq!(target.elements.len(), 1);
    assert!(matches!(
        &target.elements[0].target,
        ListAssignmentElementTarget::Reference(ReferenceTarget::Variable { name, .. })
            if name == "x"
    ));
    assert!(matches!(value.as_ref(), Expr::Variable(name, _) if name == "x"));
}

#[test]
fn parser_accepts_variable_root_array_assignment_and_unset() {
    let program = parser::parse(
        "<?php $items[null] = \"value\"; $items[] += 2; $items[0][\"nested\"] = 3; unset($items[null], $items[0][\"nested\"], $items);",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 4);

    let Statement::ArrayAssign {
        target, op, value, ..
    } = &program.statements[0]
    else {
        panic!("expected array assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::Assign);
    assert_eq!(target.dimensions.len(), 1);
    assert!(matches!(target.dimensions[0].as_ref(), Some(Expr::Null(_))));
    assert!(matches!(value, Expr::String(value, _) if value == "value"));

    let Statement::ArrayAssign { target, op, .. } = &program.statements[1] else {
        panic!("expected array append compound assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::AddAssign);
    assert_eq!(target.dimensions, vec![None]);

    let Statement::ArrayAssign { target, op, .. } = &program.statements[2] else {
        panic!("expected nested array assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::Assign);
    assert_eq!(target.dimensions.len(), 2);
    assert!(matches!(
        target.dimensions[0].as_ref(),
        Some(Expr::Int(0, _))
    ));
    assert!(
        matches!(target.dimensions[1].as_ref(), Some(Expr::String(value, _)) if value == "nested")
    );

    let Statement::Unset { targets, .. } = &program.statements[3] else {
        panic!("expected unset statement");
    };
    assert_eq!(targets.len(), 3);
    assert!(matches!(
        &targets[0],
        UnsetTarget::ArrayDim(target)
            if target.array == "items" && matches!(target.dimensions[0].as_ref(), Some(Expr::Null(_)))
    ));
    assert!(matches!(
        &targets[1],
        UnsetTarget::ArrayDim(target)
            if target.array == "items" && target.dimensions.len() == 2
    ));
    assert!(matches!(
        &targets[2],
        UnsetTarget::Variable { name, .. } if name == "items"
    ));
}

#[test]
fn parser_accepts_array_offset_compound_assignment_expressions() {
    let program = parser::parse(
        "<?php\n\
echo $items[\"count\"] += 2;\n\
($items[] .= \"x\");\n",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Assign {
            target: AssignmentTarget::ArrayDim(target),
            op: AssignmentOp::AddAssign,
            ..
        } if target.array == "items"
            && matches!(target.dimensions[0].as_ref(), Some(Expr::String(value, _)) if value == "count")
    ));

    let Statement::Expression { expression, .. } = &program.statements[1] else {
        panic!("expected grouped expression statement");
    };
    assert!(matches!(
        expression,
        Expr::Grouped { expr, .. }
            if matches!(
                expr.as_ref(),
                Expr::Assign {
                    target: AssignmentTarget::ArrayDim(target),
                    op: AssignmentOp::ConcatAssign,
                    ..
                } if target.array == "items" && target.dimensions == vec![None]
            )
    ));
}

#[test]
fn parser_rejects_unsupported_reference_forms_with_explicit_diagnostics() {
    let by_ref_return = parser::parse("<?php function &factory() { return null; }").unwrap_err();
    assert_eq!(
        by_ref_return.message,
        "by-reference return requires a variable or array element"
    );

    let temporary_assignment = parser::parse("<?php $alias =& 1;").unwrap_err();
    assert_eq!(
        temporary_assignment.message,
        "unsupported by-reference assignment target"
    );

    let dynamic_call_result_return =
        parser::parse("<?php function &factory(&$value) { $fn = 'id'; return $fn($value); }")
            .unwrap_err();
    assert_eq!(
        dynamic_call_result_return.message,
        "by-reference call-result returns are unsupported"
    );

    let recursive_return =
        parser::parse("<?php function &factory(&$value) { return factory($value); }").unwrap_err();
    assert_eq!(
        recursive_return.message,
        "recursive by-reference returns are unsupported"
    );

    let recursive_array = parser::parse("<?php $array = []; $array[] =& $array;").unwrap_err();
    assert_eq!(
        recursive_array.message,
        "recursive array references are unsupported"
    );

    parser::parse("<?php $array = [1]; $array[] =& $array[0];").unwrap();
    parser::parse("<?php $array = [1, 2]; $array[0] =& $array[1];").unwrap();

    parser::parse("<?php $array = [&$array];").unwrap();
    parser::parse("<?php $array = ['self' => &$array];").unwrap();
    parser::parse("<?php $array = [[&$array]];").unwrap();

    let recursive_array_element_literal =
        parser::parse("<?php $array = []; $array[] = [&$array];").unwrap_err();
    assert_eq!(
        recursive_array_element_literal.message,
        "recursive array references are unsupported"
    );

    let same_array_element_literal = parser::parse("<?php $array = [&$array[0]];").unwrap_err();
    assert_eq!(
        same_array_element_literal.message,
        "same-array element references are unsupported"
    );

    let same_array_element_append_literal =
        parser::parse("<?php $array = []; $array[] = [&$array[0]];").unwrap_err();
    assert_eq!(
        same_array_element_append_literal.message,
        "same-array element references are unsupported"
    );

    parser::parse("<?php $array[0][1] =& $value;").unwrap();
    parser::parse("<?php $alias =& $array[0][1];").unwrap();
    parser::parse("<?php $alias =& ($array[0][1]);").unwrap();
    parser::parse("<?php $alias =& ($array[0])[1];").unwrap();
    parser::parse("<?php $alias =& (($array)[0])[1];").unwrap();
    parser::parse("<?php $refs = [&$array[0][1]];").unwrap();
    parser::parse("<?php $refs = ['k' => &$array[0][1]];").unwrap();
    parser::parse("<?php $refs = [&($array[0])[1]];").unwrap();

    let temporary_offset_forms = [
        ("<?php $alias =& factory()[0];", "factory()[0]"),
        ("<?php $alias =& [1][0];", "[1][0]"),
        ("<?php $alias =& ($value + 1)[0];", "($value + 1)[0]"),
        ("<?php $refs = [&factory()[0]];", "factory()[0]"),
        ("<?php $refs = [&[1][0]];", "[1][0]"),
    ];
    for (source, target) in temporary_offset_forms {
        assert_reference_lvalue_diagnostic(
            source,
            "temporary array offset references are unsupported",
            target,
        );
    }
}

fn assert_reference_lvalue_diagnostic(source: &str, message: &str, target: &str) {
    let error = parser::parse(source).unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Fatal);
    assert_eq!(error.message, message);
    let span = error
        .span
        .expect("reference diagnostic should be source-spanned");
    let byte_start = source
        .find(target)
        .unwrap_or_else(|| panic!("test source should contain target {target}"));
    let byte_end = byte_start + target.len();
    assert_eq!(span.byte_start, byte_start);
    assert_eq!(span.byte_end, byte_end);
    assert_eq!(&source[span.byte_start..span.byte_end], target);
    assert_eq!(
        span.line,
        source[..byte_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    );
    assert_eq!(
        span.column,
        source[..byte_start]
            .rsplit('\n')
            .next()
            .expect("source prefix should have a final line")
            .chars()
            .count()
            + 1
    );
}

#[test]
fn parser_accepts_grouped_direct_and_single_dim_reference_targets() {
    let program =
        parser::parse("<?php $alias =& ($value); $slot =& ($items)[0]; $same =& ($items[1]);")
            .unwrap();

    assert!(matches!(
        &program.statements[0],
        Statement::AssignRef { name, .. } if name == "alias"
    ));
    assert!(matches!(
        &program.statements[1],
        Statement::AssignRef { name, .. } if name == "slot"
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::AssignRef { name, .. } if name == "same"
    ));
}

#[test]
fn parser_accepts_by_reference_return_call_assignment_sources() {
    let program = parser::parse(
        "<?php function &id(&$value) { return $value; } $alias =& id($value); $value_alias =& factory(); var_dump($items[0] =& returnsVal());",
    )
    .unwrap();
    assert!(program.functions[0].return_by_ref);
    let Statement::AssignRef { name, source, .. } = &program.statements[0] else {
        panic!("expected by-reference assignment");
    };
    assert_eq!(name, "alias");
    assert!(matches!(source, Expr::Call { name, .. } if name == "id"));

    let Statement::AssignRef { name, source, .. } = &program.statements[1] else {
        panic!("expected value-returning call by-reference assignment");
    };
    assert_eq!(name, "value_alias");
    assert!(matches!(source, Expr::Call { name, .. } if name == "factory"));

    let Statement::Call {
        name, arguments, ..
    } = &program.statements[2]
    else {
        panic!("expected var_dump call");
    };
    assert_eq!(name, "var_dump");
    assert!(matches!(
        &arguments[0],
        Expr::AssignRef {
            target: AssignmentTarget::ArrayDim(target),
            source,
            ..
        } if target.array == "items"
            && matches!(source.as_ref(), Expr::Call { name, .. } if name == "returnsval")
    ));
}

#[test]
fn parser_accepts_by_reference_return_call_result_chains() {
    let program = parser::parse(
        "<?php function &id(&$value) { return $value; } function &chain(&$value) { return id($value); } function &fallback() { return make_value(); }",
    )
    .unwrap();

    assert_eq!(program.functions.len(), 3);
    assert!(program
        .functions
        .iter()
        .all(|function| function.return_by_ref));

    let Statement::Return {
        value: Some(Expr::Call { name, .. }),
        ..
    } = &program.functions[1].body[0]
    else {
        panic!("expected chained call return");
    };
    assert_eq!(name, "id");

    let Statement::Return {
        value: Some(Expr::Call { name, .. }),
        ..
    } = &program.functions[2].body[0]
    else {
        panic!("expected value call return");
    };
    assert_eq!(name, "make_value");
}

#[test]
fn parser_accepts_reference_array_literal_values() {
    let program = parser::parse("<?php $items = [&$value, 'k' => &$source[0]];").unwrap();
    let Statement::Assign { value, .. } = &program.statements[0] else {
        panic!("expected assignment statement");
    };
    let Expr::Array { elements, .. } = value else {
        panic!("expected array literal");
    };
    assert_eq!(elements.len(), 2);
    assert!(matches!(
        &elements[0].value,
        ArrayElementValue::Reference(ReferenceTarget::Variable { name, .. }) if name == "value"
    ));
    assert!(matches!(
        &elements[1].value,
        ArrayElementValue::Reference(ReferenceTarget::ArrayDim(target))
            if target.array == "source" && target.dimensions.len() == 1
    ));
}

#[test]
fn parser_accepts_whitespace_prelude_and_reference_array_entries() {
    let program = parser::parse("\n\t <?php $items = [&$a, \"k\" => &$b];").unwrap();
    let Statement::Assign {
        name,
        value: Expr::Array { elements, .. },
        ..
    } = &program.statements[0]
    else {
        panic!("expected array assignment");
    };

    assert_eq!(name, "items");
    assert_eq!(elements.len(), 2);
    assert!(matches!(
        &elements[0].value,
        ArrayElementValue::Reference(ReferenceTarget::Variable { name, .. }) if name == "a"
    ));
    assert!(matches!(&elements[1].key, Some(Expr::String(key, _)) if key == "k"));
    assert!(matches!(
        &elements[1].value,
        ArrayElementValue::Reference(ReferenceTarget::Variable { name, .. }) if name == "b"
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
fn parser_accepts_null_coalescing_as_right_associative_expression() {
    let program = parser::parse("<?php echo $a ?? $b ?? \"fallback\";").unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::Binary {
        op: BinaryOp::Coalesce,
        left,
        right,
        ..
    } = &expressions[0]
    else {
        panic!("expected null coalescing expression");
    };
    assert!(matches!(left.as_ref(), Expr::Variable(name, _) if name == "a"));
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Coalesce,
            left,
            right,
            ..
        } if matches!(left.as_ref(), Expr::Variable(name, _) if name == "b")
            && matches!(right.as_ref(), Expr::String(value, _) if value == "fallback")
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
fn parser_accepts_class_constant_metadata_and_fetches() {
    let program = parser::parse(
        "<?php
class Sample {
    const A = 1, B = [\"x\" => 2];
    public const Label = \"ok\";

    public static function value() { return self::A; }
}
$haystack = [Sample::A];
$map = [Sample::A => Sample::B];
echo \\Sample::Label;",
    )
    .unwrap();

    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].constants.len(), 3);
    assert_eq!(program.classes[0].constants[0].name, "A");
    assert_eq!(
        program.classes[0].constants[0].visibility,
        PropertyVisibility::Public
    );
    assert!(matches!(
        program.classes[0].constants[0].value,
        Expr::Int(1, _)
    ));
    assert!(matches!(
        program.classes[0].constants[1].value,
        Expr::Array { ref elements, .. } if elements.len() == 1
    ));
    assert!(matches!(
        program.classes[0].methods[0].body[0],
        Statement::Return {
            value:
                Some(Expr::ClassConstantFetch {
                    ref class_name,
                    ref name,
                    ..
                }),
            ..
        } if class_name == "self" && name == "A"
    ));
    assert!(matches!(
        &program.statements[0],
        Statement::Assign {
            value:
                Expr::Array {
                    elements,
                    ..
                },
            ..
        } if matches!(
            &elements[0].value,
            ArrayElementValue::Value(Expr::ClassConstantFetch { class_name, name, .. })
                if class_name == "Sample" && name == "A"
        )
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::Echo { expressions, .. }
            if matches!(
                &expressions[0],
                Expr::ClassConstantFetch { class_name, name, .. }
                    if class_name == "Sample" && name == "Label"
            )
    ));
}

#[test]
fn parser_accepts_in_array_class_constant_reducer_syntax() {
    let source = "<?php
$haystack = [Sample::A];
$needle = Sample::B;
var_dump(in_array($needle, $haystack, true));";
    let program = parser::parse(source).unwrap();
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn parser_rejects_unsupported_class_constant_boundaries() {
    let cases = [
        (
            "non-public",
            "<?php class Sample { private const A = 1; }",
            "non-public class constants are unsupported",
        ),
        (
            "typed",
            "<?php class Sample { public const int A = 1; }",
            "typed class constants are unsupported",
        ),
        (
            "dynamic name",
            "<?php $name = 'A'; echo Sample::{$name};",
            "class constant fetches are unsupported; class constants and enum cases require class metadata",
        ),
    ];

    for (name, source, message) in cases {
        let error = parser::parse(source).unwrap_err();
        assert_eq!(error.message, message, "{name}");
        assert_eq!(error.kind, DiagnosticKind::Fatal, "{name}");
    }
}

#[test]
fn parser_accepts_static_class_methods_as_callable_functions() {
    let program = parser::parse(
        "<?php
class Reducer {
    public static function combine($carry, $value) { return $carry + $value; }
}
echo Reducer::combine(1, 2);",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].name, "Reducer");
    assert_eq!(program.classes[0].methods.len(), 1);
    assert_eq!(program.classes[0].methods[0].name, "combine");
    assert!(program.classes[0].methods[0].is_static);
    assert_eq!(program.classes[0].methods[0].parameters.len(), 2);
    assert_eq!(program.functions.len(), 0);
    assert_eq!(program.statements.len(), 1);
    assert!(matches!(
        &program.statements[0],
        Statement::Echo { expressions, .. }
            if matches!(
                &expressions[0],
                Expr::Call { name, arguments, .. }
                    if name == "Reducer::combine" && arguments.len() == 2
            )
    ));
}

#[test]
fn parser_accepts_declared_static_property_reads_and_writes() {
    let program = parser::parse(
        "<?php
class Counter {
    public static $value = 1, $label;
}
Counter::$value = 5;
echo Counter::$value;",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].static_properties.len(), 2);
    assert_eq!(program.classes[0].static_properties[0].name, "value");
    assert_eq!(program.classes[0].static_properties[1].name, "label");

    let Some(Expr::Int(1, _)) = &program.classes[0].static_properties[0].value else {
        panic!("expected static property default expression");
    };
    assert!(program.classes[0].static_properties[1].value.is_none());

    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected static property assignment statement");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target:
                AssignmentTarget::StaticProperty {
                    class_name,
                    name,
                    ..
                },
            ..
        } if class_name == "Counter" && name == "value"
    ));

    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected static property echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::StaticPropertyFetch {
            class_name,
            name,
            ..
        } if class_name == "Counter" && name == "value"
    ));
}

#[test]
fn parser_accepts_static_property_null_coalescing_assignment() {
    let program = parser::parse(
        "<?php
class Counter {
    public static $value;
}
Counter::$value ??= 5;
echo Counter::$value ??= 6;",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected static property null coalescing assignment statement");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target:
                AssignmentTarget::StaticProperty {
                    class_name,
                    name,
                    ..
                },
            op: AssignmentOp::CoalesceAssign,
            ..
        } if class_name == "Counter" && name == "value"
    ));

    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected static property null coalescing assignment echo");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Assign {
            target:
                AssignmentTarget::StaticProperty {
                    class_name,
                    name,
                    ..
                },
            op: AssignmentOp::CoalesceAssign,
            ..
        } if class_name == "Counter" && name == "value"
    ));
}

#[test]
fn parser_accepts_property_and_static_property_increment_targets() {
    let program = parser::parse(
        "<?php
class Counter {
    public static $value = 1;
}
$object = new stdClass;
$object->value++;
++$object->value;
Counter::$value--;
--Counter::$value;
",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 5);

    let Statement::Expression { expression, .. } = &program.statements[1] else {
        panic!("expected postfix property increment expression statement");
    };
    assert!(matches!(
        expression,
        Expr::IncDec {
            target: IncDecTarget::Property { name, .. },
            op: IncDecOp::Increment,
            result: IncDecResult::Post,
            ..
        } if name == "value"
    ));

    let Statement::Increment { target, op, .. } = &program.statements[2] else {
        panic!("expected prefix property increment statement");
    };
    assert!(matches!(target, IncDecTarget::Property { name, .. } if name == "value"));
    assert_eq!(*op, IncDecOp::Increment);

    let Statement::Expression { expression, .. } = &program.statements[3] else {
        panic!("expected postfix static-property decrement expression statement");
    };
    assert!(matches!(
        expression,
        Expr::IncDec {
            target:
                IncDecTarget::StaticProperty {
                    class_name,
                    name,
                    ..
                },
            op: IncDecOp::Decrement,
            result: IncDecResult::Post,
            ..
        } if class_name == "Counter" && name == "value"
    ));

    let Statement::Increment { target, op, .. } = &program.statements[4] else {
        panic!("expected prefix static-property decrement statement");
    };
    assert!(matches!(
        target,
        IncDecTarget::StaticProperty {
            class_name,
            name,
            ..
        } if class_name == "Counter" && name == "value"
    ));
    assert_eq!(*op, IncDecOp::Decrement);
}

#[test]
fn parser_accepts_declared_public_instance_properties() {
    let program = parser::parse(
        "<?php
class Box {
    public $name = \"ptn\", $count = 2, $unset;
}
$box = new Box;
echo $box->name;",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].properties.len(), 3);
    assert_eq!(program.classes[0].properties[0].name, "name");
    assert_eq!(program.classes[0].properties[1].name, "count");
    assert_eq!(program.classes[0].properties[2].name, "unset");

    let Some(Expr::String(value, _)) = &program.classes[0].properties[0].value else {
        panic!("expected string property default expression");
    };
    assert_eq!(value, "ptn");
    assert!(matches!(
        program.classes[0].properties[1].value,
        Some(Expr::Int(2, _))
    ));
    assert!(program.classes[0].properties[2].value.is_none());
    assert_eq!(
        program.classes[0].properties[0].visibility,
        PropertyVisibility::Public
    );
}

#[test]
fn parser_accepts_declared_non_public_instance_properties() {
    let program = parser::parse(
        "<?php
class Box {
    private $secret = 4;
    protected $guarded;
    public $public = 1;
}
",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].properties.len(), 3);
    assert_eq!(
        program.classes[0].properties[0].visibility,
        PropertyVisibility::Private
    );
    assert_eq!(
        program.classes[0].properties[1].visibility,
        PropertyVisibility::Protected
    );
    assert_eq!(
        program.classes[0].properties[2].visibility,
        PropertyVisibility::Public
    );
    assert!(matches!(
        program.classes[0].properties[0].value,
        Some(Expr::Int(4, _))
    ));
    assert!(program.classes[0].properties[1].value.is_none());
}

#[test]
fn parser_rejects_unsupported_non_public_methods() {
    let cases = [
        (
            "<?php class Box { private function secret() {} }",
            "non-public class methods are unsupported",
        ),
        (
            "<?php class Box { protected function guarded() {} }",
            "non-public class methods are unsupported",
        ),
    ];

    for (source, message) in cases {
        let error = parser::parse(source).unwrap_err();
        assert_eq!(error.message, message);
        assert_eq!(error.kind, DiagnosticKind::Fatal);
    }
}

#[test]
fn parser_accepts_private_instance_properties() {
    let program = parser::parse(
        "<?php
class Box {
    private $secret = \"ptn\", $unset;
    public function reveal() { return $this->secret; }
}
$box = new Box;
echo $box->reveal();",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].properties.len(), 2);
    assert_eq!(program.classes[0].properties[0].name, "secret");
    assert_eq!(
        program.classes[0].properties[0].visibility,
        PropertyVisibility::Private
    );
    assert_eq!(
        program.classes[0].properties[1].visibility,
        PropertyVisibility::Private
    );
}

#[test]
fn parser_accepts_declared_non_public_property_metadata() {
    let program = parser::parse(
        "<?php
class Box {
    private $secret = 1, $other;
    protected static $shared = 2;
}
",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].properties.len(), 2);
    assert_eq!(
        program.classes[0].properties[0].visibility,
        PropertyVisibility::Private
    );
    assert_eq!(
        program.classes[0].properties[1].visibility,
        PropertyVisibility::Private
    );
    assert_eq!(program.classes[0].static_properties.len(), 1);
    assert_eq!(
        program.classes[0].static_properties[0].visibility,
        PropertyVisibility::Protected
    );
}

#[test]
fn parser_rejects_non_public_methods_before_visibility_dispatch() {
    let error = parser::parse("<?php class Box { private function hidden() {} }").unwrap_err();
    assert_eq!(error.message, "non-public class methods are unsupported");
}

#[test]
fn parser_accepts_instance_class_methods_and_object_callables() {
    let program = parser::parse(
        "<?php
class Worker {
    public function run($value) { return $value + 1; }
}
$worker = new Worker();
echo $worker->run(3);
call_user_func([$worker, \"run\"], 4);",
    )
    .unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].methods.len(), 1);
    assert_eq!(program.classes[0].methods[0].name, "run");
    assert!(!program.classes[0].methods[0].is_static);
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(
        &program.statements[0],
        Statement::Assign {
            value: Expr::NewObject { class_name, .. },
            ..
        } if class_name == "Worker"
    ));
    assert!(matches!(
        &program.statements[1],
        Statement::Echo { expressions, .. }
            if matches!(&expressions[0], Expr::MethodCall { name, .. } if name == "run")
    ));
    assert!(matches!(
        &program.statements[2],
        Statement::Call { name, arguments, .. }
            if name == "call_user_func" && arguments.len() == 2
    ));
}

#[test]
fn parser_accepts_simple_class_inheritance_metadata() {
    let program = parser::parse(
        "<?php
class Base {
    public function label($value) { return $value; }
}

class Child extends Base {
    public function own($value) { return $this->label($value); }
}",
    )
    .unwrap();

    assert_eq!(program.classes.len(), 2);
    assert_eq!(program.classes[0].name, "Base");
    assert_eq!(program.classes[0].parent_name, None);
    assert_eq!(program.classes[1].name, "Child");
    assert_eq!(program.classes[1].parent_name.as_deref(), Some("Base"));
    assert_eq!(program.classes[1].methods.len(), 1);
}

#[test]
fn parser_rejects_class_like_declarations_with_explicit_diagnostics() {
    let cases = [
        (
            "enum",
            "<?php\nenum Sample { case A; }",
            "enum declarations are unsupported",
        ),
        (
            "interface",
            "<?php\ninterface Sample {}",
            "interface declarations are unsupported",
        ),
        (
            "trait",
            "<?php\ntrait Sample {}",
            "trait declarations are unsupported",
        ),
    ];

    for (name, source, message) in cases {
        let error = parser::parse(source).unwrap_err();
        assert_eq!(error.message, message, "{name}");
        assert_eq!(error.kind, DiagnosticKind::Fatal, "{name}");
        assert_eq!(error.span.unwrap().line, 2, "{name}");
    }
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
fn parser_accepts_braced_array_interpolated_strings() {
    let program = parser::parse(
        "<?php echo \"name={$name} item={$items['name']} dynamic={$items[$key]} zero={$items[0]}\\n\";",
    )
    .unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::InterpolatedString(parts, _) = &expressions[0] else {
        panic!("expected interpolated string");
    };
    assert_eq!(
        parts,
        &vec![
            StringPart::Literal("name=".to_string()),
            StringPart::Variable("name".to_string()),
            StringPart::Literal(" item=".to_string()),
            StringPart::ArrayAccess {
                array: "items".to_string(),
                indices: vec![StringInterpolationIndex::String("name".to_string())],
            },
            StringPart::Literal(" dynamic=".to_string()),
            StringPart::ArrayAccess {
                array: "items".to_string(),
                indices: vec![StringInterpolationIndex::Variable("key".to_string())],
            },
            StringPart::Literal(" zero=".to_string()),
            StringPart::ArrayAccess {
                array: "items".to_string(),
                indices: vec![StringInterpolationIndex::Int(0)],
            },
            StringPart::Literal("\n".to_string()),
        ]
    );
}

#[test]
fn parser_accepts_simple_array_and_legacy_dollar_brace_interpolation() {
    let program =
        parser::parse("<?php echo \"item=$items[$key] bare=$items[name] legacy=${name}!\\n\";")
            .unwrap();
    let Statement::Echo { expressions, .. } = &program.statements[0] else {
        panic!("expected echo statement");
    };
    let Expr::InterpolatedString(parts, _) = &expressions[0] else {
        panic!("expected interpolated string");
    };
    assert_eq!(
        parts,
        &vec![
            StringPart::Literal("item=".to_string()),
            StringPart::ArrayAccess {
                array: "items".to_string(),
                indices: vec![StringInterpolationIndex::Variable("key".to_string())],
            },
            StringPart::Literal(" bare=".to_string()),
            StringPart::ArrayAccess {
                array: "items".to_string(),
                indices: vec![StringInterpolationIndex::String("name".to_string())],
            },
            StringPart::Literal(" legacy=".to_string()),
            StringPart::LegacyDollarBraceVariable("name".to_string()),
            StringPart::Literal("!\n".to_string()),
        ]
    );
}

#[test]
fn parser_rejects_unsupported_braced_property_interpolation() {
    let error = parser::parse("<?php echo \"name={$object->name}\\n\";").unwrap_err();
    assert_eq!(error.message, "complex string interpolation is unsupported");
    assert_eq!(error.span.unwrap().line, 1);
}

#[test]
fn parser_rejects_alternative_offset_syntax_in_braced_interpolation_as_parse_error() {
    let error = parser::parse("<?php \"{$g{'h'}}\";").unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::ParseError);
    assert_eq!(
        error.message,
        "syntax error, unexpected token \"{\", expecting \"->\" or \"?->\" or \"[\""
    );
    assert_eq!(error.span.unwrap().line, 1);
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
fn phpc_renders_alternative_offset_in_interpolation_as_php_parse_error() {
    let root = temp_dir("ptn-phpc-alternative-offset-interpolation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("alternative-offset-interpolation.php");
    fs::write(&input, "<?php\n\"{$g{'h'}}\";\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Parse error: syntax error, unexpected token \"{{\", expecting \"->\" or \"?->\" or \"[\" in {} on line 2\n",
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
        .find("static PTN_UNUSED void ptn_echo(PtnRuntime *runtime, PtnValue value, size_t line)")
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
fn compile_object_to_string_conversion_to_native_binary() {
    let root = temp_dir("ptn-native-object-to-string-conversion");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("object-to-string-conversion.php");
    let output = root.join("object-to-string-conversion-bin");
    fs::write(
        &input,
        "<?php
class StringCapable {
    public function __toString() {
        return \"Hello, world\";
    }
}

$value = new StringCapable();
var_dump(strlen($value));
var_dump(strlen(\"$value\"));
echo $value, \"\\n\";
echo \"prefix:$value\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(12)\nint(12)\nHello, world\nprefix:Hello, world\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("runtime.method_dispatch = ptn_call_declared_method;"));
    assert!(c_source.contains("ptn_value_to_string_operand_with_runtime"));
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
fn compile_print_expression_contexts_to_native_binary() {
    let root = temp_dir("ptn-native-print-expression-contexts");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("print-expression-contexts.php");
    let output = root.join("print-expression-contexts-bin");
    fs::write(
        &input,
        "<?php
$result = print \"A\";
echo \"|result=\", $result, \"\\n\";
echo \"echo:\", print \"B\", \"|\\n\";
$right = 2 + print \"C\";
echo \"|right=\", $right, \"\\n\";
$left = (print \"D\") + 2;
echo \"|left=\", $left, \"\\n\";
$paren = print(\"E\");
echo \"|paren=\", $paren, \"\\n\";
print \"F\" . \"G\";
echo \"|\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "A|result=1\n\
echo:B1|\n\
C|right=3\n\
D|left=3\n\
E|paren=1\n\
FG|\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_echo("));
    assert!(c_source.contains("ptn_int(1)"));
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}NULL\nNULL\n",
            undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
echo strcasecmp(\"AbC\", \"aBc\"), \" \", strcasecmp(\"abc\", \"ABD\"), \" \", strcasecmp(\"A\" . chr(0) . \"Z\", \"a\" . chr(0) . \"y\"), \" \", strcasecmp(\"\\x80\", \"A\"), \"\\n\";\n\
echo str_rot13(\"abc\"), \" \", substr(\"abcdef\", 2, 3), \" \", bin2hex(\"Az\"), \" \", quotemeta(\"a.b\"), \" \", chunk_split(\"abcd\", 2, \"|\"), \"\\n\";\n\
echo bin2hex(strtolower(\"Az\" . chr(0) . \"Q\" . chr(255))), \" \", bin2hex(strtoupper(\"az\" . chr(0) . \"q\" . chr(255))), \" \", bin2hex(lcfirst(\"Az\" . chr(0) . \"Q\" . chr(255))), \"\\n\";\n\
echo strpos(\"abcabc\", \"bc\"), \" \", stripos(\"AbCaBc\", \"bc\"), \" \", strrpos(\"abcabc\", \"bc\"), \" \", strripos(\"AbCaBc\", \"BC\"), \"\\n\";\n\
var_dump(strpos(\"abc\", \"z\"), strpos(\"abc\", \"\", 2), strrpos(\"abcabc\", \"\", -1));\n\
echo strstr(\"abcabc\", \"bc\"), \"|\", strstr(\"abcabc\", \"bc\", true), \"|\", stristr(\"AbCaBc\", \"BC\"), \"\\n\";\n\
echo substr_count(\"abcabcabc\", \"abc\"), \" \", substr_count(\"abcabcabc\", \"abc\", 1), \" \", substr_count(\"abcabcabc\", \"abc\", 0, -3), \"\\n\";\n\
echo bin2hex(strstr(\"A\" . chr(0) . \"BC\", chr(0))), \" \", strpos(\"A\" . chr(0) . \"BC\", chr(0)), \"\\n\";\n\
echo strip_tags(\"<b>x</b>\"), \" \", quoted_printable_decode(\"=41\"), \" \", soundex(\"Robert\"), \" \", ord(\"A\"), \" \", bindec(\"101\"), \" \", hexdec(\"ff\"), \" \", octdec(\"10\"), \"\\n\";\n\
echo bin2hex(strip_tags(\"<b>A</b>\" . chr(0) . \"<i>B</i>\")), \" \", soundex(\"A\" . chr(0) . \"B\"), \"\\n\";\n\
echo str_pad(\"x\", 4, \"ab\", STR_PAD_LEFT), \" \", str_pad(\"x\", 4, \"ab\", STR_PAD_RIGHT), \" \", str_pad(\"x\", 5, \"ab\", STR_PAD_BOTH), \"\\n\";\n\
echo str_repeat(\"xy\", 3), \"|\", str_repeat(\"z\", 0), \"|\", chunk_split(str_repeat(\"X\", 6), 3, \"|\"), \"\\n\";\n\
echo trim(\" \\tHi\\r\\n\"), \"|\", ltrim(\"==left\", \"=\"), \"|\", rtrim(\"right!!\", \"!\"), \"\\n\";\n\
echo md5(\"\"), \" \", sha1(\"\"), \"\\n\";\n\
var_dump(strncmp(\"abc\", \"abd\", 3), strncmp(\"abc\", \"abd\", 2), strncmp(\"a\" . chr(0) . \"c\", \"a\" . chr(0) . \"d\", 3), strncmp(\"a\", \"a\" . chr(0), 2), strncmp(12345, \"123\", \"3\"), function_exists(\"strncmp\"), function_exists(\"STRNCMP\"));\n\
try { strncmp(\"a\", \"b\", -1); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(crc32(\"string_val1234\"), crc32(\"a\" . chr(0) . \"b\"), crc32(12345), function_exists(\"CRC32\"));\n\
var_dump(strlen(12345), bin2hex(255), substr(12345, 1, 2), strtolower(true), strtoupper(false), function_exists(\"str_pad\"), function_exists(\"STRCASECMP\"), defined(\"STR_PAD_BOTH\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "6 -1 1 1 1\n0 -1 1 1\nnop cde 417a a\\.b ab|cd|\n\
617a0071ff 415a0051ff 617a0051ff\n\
1 1 4 4\n\
bool(false)\nint(2)\nint(5)\n\
bcabc|a|bCaBc\n\
3 2 2\n\
004243 1\n\
x A R163 65 5 255 8\n\
4142 A100\n\
abax xaba abxab\n\
xyxyxy||XXX|XXX|\n\
Hi|left|right\n\
d41d8cd98f00b204e9800998ecf8427e da39a3ee5e6b4b0d3255bfef95601890afd80709\n\
int(-1)\nint(0)\nint(-1)\nint(-1)\nint(0)\nbool(true)\nbool(true)\n\
strncmp(): Argument #3 ($length) must be greater than or equal to 0\n\
int(256895812)\nint(367556721)\nint(3421846044)\nbool(true)\n\
int(5)\nstring(6) \"323535\"\nstring(2) \"23\"\nstring(1) \"1\"\nstring(0) \"\"\n\
bool(true)\nbool(true)\nbool(true)\n"
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
        "ptn_internal_strcasecmp",
        "ptn_internal_str_contains",
        "ptn_internal_str_starts_with",
        "ptn_internal_str_ends_with",
        "ptn_internal_strncmp",
        "ptn_internal_strpos",
        "ptn_internal_stripos",
        "ptn_internal_strrpos",
        "ptn_internal_strripos",
        "ptn_internal_strstr",
        "ptn_internal_stristr",
        "ptn_internal_substr_count",
        "ptn_internal_str_pad",
        "ptn_internal_strtolower",
        "ptn_internal_strtoupper",
        "ptn_internal_trim",
        "ptn_internal_ltrim",
        "ptn_internal_rtrim",
        "ptn_internal_strrev",
        "ptn_internal_ucfirst",
        "ptn_internal_lcfirst",
        "ptn_internal_quotemeta",
        "ptn_internal_chunk_split",
        "ptn_internal_str_repeat",
        "ptn_internal_strip_tags",
        "ptn_internal_crc32",
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
            body.contains("ptn_value_to_string_operand")
                || body.contains("ptn_internal_expect_string_arg")
                || body.contains("ptn_internal_trim_named")
                || body.contains("ptn_internal_strpos_named")
                || body.contains("ptn_internal_strrpos_named")
                || body.contains("ptn_internal_strstr_named"),
            "{function} should use the direct string operand helper"
        );
        assert!(
            !body.contains("ptn_value_to_string(args"),
            "{function} should not convert direct argument expressions unconditionally"
        );
    }

    for expected_call in [
        "ptn_rot13_string(string.data, string.len)",
        "ptn_first_char_case_string(string.data, string.len, 1)",
        "ptn_first_char_case_string(string.data, string.len, 0)",
        "ptn_ascii_case_string(string.data, string.len, 0)",
        "ptn_ascii_case_string(string.data, string.len, 1)",
        "ptn_trim_string_value(input, charlist, trim_left, trim_right)",
        "ptn_compare_string_prefix_bytes(",
        "ptn_string_buffer_append_repeated_pattern(&output, pad_string, left_len)",
        "ptn_quotemeta_string(input.data, input.len, &output_len)",
        "ptn_strip_tags_string(input.data, input.len, &output_len)",
        "ptn_crc32_bytes((const unsigned char *)input.data, input.len)",
        "ptn_dirname_string(path.data, path.len, &dirname_len)",
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
        "static char *ptn_first_char_case_string(",
        "static char *ptn_ascii_case_string(",
        "static int ptn_compare_string_prefix_bytes(",
        "static PtnValue ptn_internal_str_pad(",
        "static char *ptn_quotemeta_string(",
        "static char *ptn_chunk_split_string(",
        "static char *ptn_strip_tags_string(",
        "static char *ptn_dirname_string(",
        "static char *ptn_quoted_printable_decode_string(",
        "static char *ptn_addslashes_string(",
        "static char *ptn_stripslashes_string(",
        "static PtnValue ptn_base_string_to_number(",
    ] {
        let body = generated_c_static_function_body(&c_source, marker);
        assert!(
            !body.contains("strlen("),
            "{marker} should consume caller-provided lengths instead of rescanning"
        );
    }

    let strip_tags_body =
        generated_c_static_function_body(&c_source, "static char *ptn_strip_tags_string(");
    assert!(
        strip_tags_body.contains("size_t len, size_t *output_len_out"),
        "strip_tags helper should report explicit output length"
    );

    let soundex_body =
        generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_soundex(");
    assert!(
        soundex_body.contains("first < string.len") && soundex_body.contains("i < string.len"),
        "soundex should iterate using the known operand length"
    );
}

#[test]
fn compile_trim_ltrim_rtrim_phpt_shapes_to_native_binary() {
    let root = temp_dir("ptn-native-trim-ltrim-rtrim-phpt-shapes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("trim-ltrim-rtrim.php");
    let output = root.join("trim-ltrim-rtrim-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing trim/ltrim/rtrim() : basic functionality ***\\n\";\n\
$text = \"  \\t\\r\\n\\0\\x0B  ---These are a few words---  \\t\\r\\n\\0\\x0B  \";\n\
$left = \" \\t\\r\\n\\0\\x0B ---These are a few words---  \";\n\
$right = \"---These are a few words---  \\t\\r\\n\\0\\x0B  \";\n\
$hello = \"!===Hello World===!\";\n\
$alpha_left = \"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\";\n\
$alpha_right = \"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\";\n\
\n\
var_dump(trim($text));\n\
var_dump(ltrim($left));\n\
var_dump(rtrim($right));\n\
var_dump(trim($hello, \"=!\"));\n\
var_dump(ltrim($hello, \"!oleH=\"));\n\
var_dump(rtrim($hello, \"!dlWro=\"));\n\
var_dump(ltrim($alpha_left, \"A..Z\"));\n\
var_dump(rtrim($alpha_right, \"A..Z\"));\n\
var_dump(\"ABC\\x50\\xC1\" === trim(\"ABC\\x50\\xC1\\x60\\x90\", \"\\x51..\\xC0\"));\n\
var_dump(\"ABC\" === trim(\"ABC\\x50\\xC1\\x60\\x90\", \"\\x50..\\xC1\"));\n\
var_dump(\" \\0\\t\\nABC \\0\\t\\n\" === trim(\" \\0\\t\\nABC \\0\\t\\n\", \"\"));\n\
var_dump(function_exists(\"trim\"), function_exists(\"LTRIM\"), function_exists(\"rtrim\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "*** Testing trim/ltrim/rtrim() : basic functionality ***\n\
string(27) \"---These are a few words---\"\n\
string(29) \"---These are a few words---  \"\n\
string(27) \"---These are a few words---\"\n\
string(11) \"Hello World\"\n\
string(10) \" World===!\"\n\
string(10) \"!===Hello \"\n\
string(10) \"0123456789\"\n\
string(10) \"0123456789\"\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_str_pad_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-str-pad");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-pad.php");
    let output = root.join("str-pad-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(STR_PAD_LEFT, STR_PAD_RIGHT, STR_PAD_BOTH);\n\
var_dump(str_pad(\"pad\", 6));\n\
var_dump(str_pad(\"pad\", 6, \"01\", STR_PAD_LEFT));\n\
var_dump(str_pad(\"pad\", 6, \"01\", STR_PAD_RIGHT));\n\
var_dump(str_pad(\"pad\", 8, \"01\", STR_PAD_BOTH));\n\
var_dump(bin2hex(str_pad(\"A\" . chr(0) . \"B\", 6, chr(255), STR_PAD_BOTH)));\n\
try { str_pad(\"pad\", 6, \"\"); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { str_pad(\"pad\", 6, \".\", 99); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists(\"str_pad\"), function_exists(\"STR_PAD\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(0)\nint(1)\nint(2)\n\
string(6) \"pad   \"\n\
string(6) \"010pad\"\n\
string(6) \"pad010\"\n\
string(8) \"01pad010\"\n\
string(12) \"ff410042ffff\"\n\
str_pad(): Argument #3 ($pad_string) must not be empty\n\
str_pad(): Argument #4 ($pad_type) must be STR_PAD_LEFT, STR_PAD_RIGHT, or STR_PAD_BOTH\n\
bool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_strrev_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-strrev");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strrev.php");
    let output = root.join("strrev-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(strrev(\"Hello\"));\n\
var_dump(bin2hex(strrev(\"A\" . chr(0) . \"B\")));\n\
var_dump(strrev(12345));\n\
var_dump(function_exists(\"strrev\"), function_exists(\"STRREV\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"olleH\"\nstring(6) \"420041\"\nstring(5) \"54321\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_ucfirst_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-ucfirst");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("ucfirst.php");
    let output = root.join("ucfirst-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(ucfirst(\"hello\"));\n\
var_dump(ucfirst(\"Hello\"));\n\
var_dump(ucfirst(\"1hello\"));\n\
var_dump(bin2hex(ucfirst(\"a\" . chr(0) . \"z\")));\n\
var_dump(ucfirst(12345));\n\
var_dump(function_exists(\"ucfirst\"), function_exists(\"UCFIRST\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"Hello\"\n\
string(5) \"Hello\"\n\
string(6) \"1hello\"\n\
string(6) \"41007a\"\n\
string(5) \"12345\"\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_lcfirst_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-lcfirst");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("lcfirst.php");
    let output = root.join("lcfirst-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(lcfirst(\"Hello\"));\n\
var_dump(lcfirst(\"hello\"));\n\
var_dump(lcfirst(\"1Hello\"));\n\
var_dump(bin2hex(lcfirst(\"A\" . chr(0) . \"Z\")));\n\
var_dump(lcfirst(12345));\n\
var_dump(function_exists(\"lcfirst\"), function_exists(\"LCFIRST\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"hello\"\n\
string(5) \"hello\"\n\
string(6) \"1Hello\"\n\
string(6) \"61005a\"\n\
string(5) \"12345\"\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_dirname_edge_paths_and_type_error_to_native_binary() {
    let root = temp_dir("ptn-native-dirname-edge-paths");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dirname-edge-paths.php");
    let output = root.join("dirname-edge-paths-bin");
    fs::write(
        &input,
        "<?php\n\
$paths = [\"\", \"c:\\\\test\\\\afile\", \"c://test//afile\", \"/foo\" . chr(0) . \"bar/t.gz\", \"/foo\" . chr(0) . \"bar/\"];\n\
foreach ($paths as $path) {\n\
    var_dump(dirname($path));\n\
}\n\
try {\n\
    dirname([]);\n\
} catch (\\TypeError $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"string(0) \"\"\n\
string(1) \".\"\n\
string(8) \"c://test\"\n\
string(8) \"/foo\0bar\"\n\
string(1) \"/\"\n\
dirname(): Argument #1 ($path) must be of type string, array given\n"
            .to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_chunk_split_str_repeat_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-chunk-split-str-repeat-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chunk-split-str-repeat.php");
    let output = root.join("chunk-split-str-repeat-bin");
    fs::write(
        &input,
        "<?php\n\
echo chunk_split('abc', 1, '-').\"\\n\";\n\
echo chunk_split('foooooooooooooooo', 5).\"\\n\";\n\
echo chunk_split(str_repeat('X', 2*76)).\"\\n\";\n\
echo chunk_split(\"test\", 10, \"|end\") . \"\\n\";\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "a-b-c-\n\
foooo\r\n\
ooooo\r\n\
ooooo\r\n\
oo\r\n\
\n\
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\r\n\
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\r\n\
\n\
test|end\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chunk_split_empty_input_returns_binary_ending_to_native_binary() {
    let root = temp_dir("ptn-native-chunk-split-empty-binary-ending");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chunk-split-empty-binary-ending.php");
    let output = root.join("chunk-split-empty-binary-ending-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(chunk_split(\"\", 5, \"????\"));\n\
$binary = chunk_split(\"\", 5, \"|\" . chr(0));\n\
echo strlen($binary), \" \", bin2hex($binary), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(execution.stdout, b"string(4) \"????\"\n2 7c00\n".to_vec());
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_explode_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-explode-internal-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("explode.php");
    let output = root.join("explode-bin");
    fs::write(
        &input,
        "<?php\n\
$parts = explode(':', 'a:b::c');\n\
var_dump($parts);\n\
var_dump(explode(':', 'a:b', 0));\n\
var_dump(explode(':', 'a:b::c', 3));\n\
var_dump(explode(':', 'a:b::c', -1));\n\
var_dump(explode(':', 'abc', -1));\n\
var_dump(explode('a', ''));\n\
echo bin2hex(explode(chr(0), 'a' . chr(0) . 'b' . chr(0) . 'c')[1]), \"\\n\";\n\
try { explode('', 'abc'); } catch (\\ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('explode'), function_exists('EXPLODE'));\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(4) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  string(0) \"\"\n",
            "  [3]=>\n",
            "  string(1) \"c\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(3) \"a:b\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  string(2) \":c\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  string(0) \"\"\n",
            "}\n",
            "array(0) {\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(0) \"\"\n",
            "}\n",
            "62\n",
            "explode(): Argument #1 ($separator) must not be empty, use str_split() to split a string into characters\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PtnValue ptn_internal_explode("));
    let body = generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_explode(");
    assert!(body.contains("ptn_internal_expect_string_arg"));
    assert!(!body.contains("strtok"));
}

#[test]
fn compile_str_split_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-str-split");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-split.php");
    let output = root.join("str-split-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(str_split(\"abcd\", 2));\n\
var_dump(str_split(\"\"));\n\
$binary = str_split(\"A\" . chr(0) . \"B\", 2);\n\
echo bin2hex($binary[0]), \":\", bin2hex($binary[1]), \"\\n\";\n\
try { str_split(\"abc\", 0); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists(\"str_split\"), function_exists(\"STR_SPLIT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [0]=>\n  string(2) \"ab\"\n  [1]=>\n  string(2) \"cd\"\n}\narray(0) {\n}\n4100:42\nstr_split(): Argument #2 ($length) must be greater than 0\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PtnValue ptn_internal_str_split("));
    let body =
        generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_str_split(");
    assert!(body.contains("ptn_internal_expect_string_arg"));
    assert!(body.contains("ptn_internal_expect_integer_arg"));
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
fn compile_string_internals_reject_non_string_values_to_native_binary() {
    let root = temp_dir("ptn-native-string-internal-type-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-internal-type-diagnostics.php");
    let output = root.join("string-internal-type-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
try { strlen([]); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { bin2hex([]); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { quotemeta([]); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { chunk_split('abc', 2, []); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { str_contains('abc', []); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { addslashes([]); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { ord([]); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { strlen(new stdClass); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { ord(new stdClass); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { dirname(new stdClass); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { strlen(function () {}); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
$bytes = 'a' . chr(0) . 'b';\n\
echo strlen($bytes), ' ', bin2hex($bytes), ' ', ord($bytes[1]), \"\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"strlen(): Argument #1 ($string) must be of type string, array given\n\
bin2hex(): Argument #1 ($string) must be of type string, array given\n\
quotemeta(): Argument #1 ($string) must be of type string, array given\n\
chunk_split(): Argument #3 ($separator) must be of type string, array given\n\
str_contains(): Argument #2 ($needle) must be of type string, array given\n\
addslashes(): Argument #1 ($string) must be of type string, array given\n\
ord(): Argument #1 ($character) must be of type string, array given\n\
strlen(): Argument #1 ($string) must be of type string, stdClass given\n\
ord(): Argument #1 ($character) must be of type string, stdClass given\n\
dirname(): Argument #1 ($path) must be of type string, stdClass given\n\
strlen(): Argument #1 ($string) must be of type string, Closure given\n\
3 610062 0\n"
            .to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_str_replace_count_and_array_string_type_errors_to_native_binary() {
    let root = temp_dir("ptn-native-str-replace-count-type-errors");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-replace-count-type-errors.php");
    let output = root.join("str-replace-count-type-errors-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(str_replace('', '', '', $count));\n\
var_dump($count);\n\
var_dump(str_replace('q', 'q', 'q', $count));\n\
var_dump($count);\n\
$fp = fopen(__FILE__, 'r');\n\
try { var_dump(str_replace($fp, 'x', 'x')); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(str_replace('x', $fp, 'x')); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(str_replace('x', 'x', $fp)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
fclose($fp);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"string(0) \"\"\n\
int(0)\n\
string(1) \"q\"\n\
int(1)\n\
str_replace(): Argument #1 ($search) must be of type array|string, resource given\n\
str_replace(): Argument #2 ($replace) must be of type array|string, resource given\n\
str_replace(): Argument #3 ($subject) must be of type array|string, resource given\n"
            .to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_double_quoted_byte_escapes_to_native_binary() {
    let root = temp_dir("ptn-native-double-quoted-byte-escapes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("double-quoted-byte-escapes.php");
    let output = root.join("double-quoted-byte-escapes-bin");
    fs::write(
        &input,
        r#"<?php
$bytes = "\x00\x0a\x7f\x80\x90\xff\377\x100";
echo strlen($bytes), " ", bin2hex($bytes), "\n";
echo ord("\xFF"), " ", ord("\377"), "\n";
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "9 000a7f8090ffff1030\n255 255\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn compile_sha1_file_binary_safe_and_error_suppression_to_native_binary() {
    let root = temp_dir("ptn-native-sha1-file-binary-safe");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sha1-file-binary-safe.php");
    let output = root.join("sha1-file-binary-safe-bin");
    fs::write(
        &input,
        "<?php\n\
$filename = __DIR__ . \"/sha1-file.dat\";\n\
$s = \"a\" . chr(0) . \"b\";\n\
var_dump(file_put_contents($filename, $s));\n\
echo sha1_file($filename), \"\\n\";\n\
echo bin2hex(sha1_file($filename, true)), \"\\n\";\n\
@unlink($filename);\n\
@unlink($filename);\n\
sha1_file($filename);\n\
echo \"done\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let stdout = String::from_utf8(execution.stdout).unwrap();
    assert!(stdout.contains("int(3)\n"));
    assert!(stdout.contains(
        "4a3dec2d1f8245280855c42db0ee4239f917fdb8\n4a3dec2d1f8245280855c42db0ee4239f917fdb8\n"
    ));
    assert!(stdout.contains("Warning: sha1_file("));
    assert!(stdout.contains(
        "sha1-file.dat): Failed to open stream: No such file or directory in ptn on line 9\n"
    ));
    assert!(!stdout.contains("Warning: unlink("));
    assert!(stdout.ends_with("done\n"));
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_file_get_contents_binary_safe_offsets_to_native_binary() {
    let root = temp_dir("ptn-native-file-get-contents");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("file-get-contents.php");
    let output = root.join("file-get-contents-bin");
    fs::write(
        &input,
        "<?php\n\
$filename = __DIR__ . \"/read.dat\";\n\
$s = \"ab\" . chr(0) . \"cdef\";\n\
var_dump(file_put_contents($filename, $s));\n\
$all = file_get_contents($filename);\n\
echo strlen($all), \" \", bin2hex($all), \"\\n\";\n\
echo bin2hex(file_get_contents($filename, false, null, 3, 2)), \"\\n\";\n\
echo bin2hex(file_get_contents($filename, false, null, -2)), \"\\n\";\n\
var_dump(file_get_contents($filename, false, null, 99, 5));\n\
try {\n\
    file_get_contents($filename, false, null, 0, -1);\n\
} catch (ValueError $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
var_dump(function_exists(\"file_get_contents\"), function_exists(\"FILE_GET_CONTENTS\"));\n\
@unlink($filename);\n\
file_get_contents($filename);\n\
echo \"done\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let stdout = String::from_utf8(execution.stdout).unwrap();
    assert!(stdout.contains("int(7)\n7 61620063646566\n6364\n6566\nstring(0) \"\"\n"));
    assert!(stdout.contains(
        "file_get_contents(): Argument #5 ($length) must be greater than or equal to 0\n"
    ));
    assert!(stdout.contains("bool(true)\nbool(true)\n"));
    assert!(stdout.contains("Warning: file_get_contents("));
    assert!(stdout.contains(
        "read.dat): Failed to open stream: No such file or directory in ptn on line 17\n"
    ));
    assert!(stdout.ends_with("done\n"));
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_highlight_string_and_empty_output_buffer_to_native_binary() {
    let root = temp_dir("ptn-native-highlight-string-ob");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("highlight-string-ob.php");
    let output = root.join("highlight-string-ob-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(function_exists(\"highlight_string\"), function_exists(\"highlight_file\"), function_exists(\"ob_get_contents\"));\n\
echo highlight_string(\"<A&>\", true), \"\\n\";\n\
highlight_string(\"A\", false);\n\
echo \"\\n\";\n\
var_dump(ob_get_contents());\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\n\
<code><span style=\"color: #000000\">\n&lt;A&amp;&gt;</span>\n</code>\n\
<code><span style=\"color: #000000\">\nA</span>\n</code>\n\
bool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_highlight_file_missing_path_to_native_binary() {
    let root = temp_dir("ptn-native-highlight-file-missing");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("highlight-file-missing.php");
    let output = root.join("highlight-file-missing-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(highlight_file(\"missing-highlight.php\", true));\n\
var_dump(ob_get_contents());\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "Warning: highlight_file(missing-highlight.php): Failed to open stream: No such file or directory in {} on line 2\n\n\
Warning: highlight_file(): Failed opening 'missing-highlight.php' for highlighting in {} on line 2\n\
bool(false)\nbool(false)\n",
            input.display(),
            input.display()
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_highlight_file_reads_source_to_native_binary() {
    let root = temp_dir("ptn-native-highlight-file-source");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("highlight-file-source.php");
    let output = root.join("highlight-file-source-bin");
    fs::write(
        &input,
        "<?php\n\
$file = __DIR__ . \"/source-to-highlight.php\";\n\
var_dump(file_put_contents($file, \"<A&>\\n\"));\n\
$result = highlight_file($file, true);\n\
var_dump(is_string($result), str_contains($result, \"&lt;A&amp;&gt;\"), ob_get_contents());\n\
var_dump(highlight_file($file, false));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(5)\n\
bool(true)\nbool(true)\nbool(false)\n\
<code><span style=\"color: #000000\">\n&lt;A&amp;&gt;\n</span>\n</code>bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_join_and_implode_to_native_binary() {
    let root = temp_dir("ptn-native-join-implode");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("join-implode.php");
    let output = root.join("join-implode-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"x\", 7, false, null, \"z\"];\n\
echo join(\"\", [1, 2, 3, 4]), \"\\n\";\n\
echo implode(\"-\", $items), \"\\n\";\n\
echo implode($items), \"\\n\";\n\
echo bin2hex(join(\"\", [\"A\", chr(0), \"B\"])), \"\\n\";\n\
var_dump(function_exists(\"join\"), function_exists(\"implode\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "1234\nx-7---z\nx7z\n410042\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PtnValue ptn_internal_implode("));
    assert!(c_source.contains("static PtnValue ptn_internal_join("));
}

#[test]
fn compile_sprintf_scalar_formats_to_native_binary() {
    let root = temp_dir("ptn-native-sprintf-scalar-formats");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sprintf-scalar-formats.php");
    let output = root.join("sprintf-scalar-formats-bin");
    fs::write(
        &input,
        "<?php\n\
echo sprintf(\"%0.3f\", 1 / 24), \"\\n\";\n\
echo sprintf(\"[%+05d] [%6.3s] [%04b] [%X] [%%]\", 7, \"abcdef\", 5, 255), \"\\n\";\n\
echo bin2hex(sprintf(\"%c\", 0)), \"\\n\";\n\
var_dump(function_exists(\"sprintf\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "0.042\n[+0007] [   abc] [0101] [FF] [%]\n00\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PtnValue ptn_internal_sprintf("));
}

#[test]
fn compile_json_encode_and_printf_to_native_binary() {
    let root = temp_dir("ptn-native-json-printf");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("json-printf.php");
    let output = root.join("json-printf-bin");
    fs::write(
        &input,
        "<?php\n\
echo json_encode(null), \"\\n\";\n\
echo json_encode(true), \"\\n\";\n\
echo json_encode([1, true, null, \"a\\nb\", \"/\"]), \"\\n\";\n\
echo json_encode([\"x\" => 1, \"two\" => false]), \"\\n\";\n\
var_dump(printf(\"%s: %s\\n\", \"flag\", json_encode(false)));\n\
var_dump(function_exists(\"json_encode\"), function_exists(\"printf\"), function_exists(\"PRINTF\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "null\n\
true\n\
[1,true,null,\"a\\nb\",\"\\/\"]\n\
{\"x\":1,\"two\":false}\n\
flag: false\n\
int(12)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PtnValue ptn_internal_json_encode("));
    assert!(c_source.contains("static PtnValue ptn_internal_printf("));
}

#[test]
fn compile_recursive_mkdir_and_directory_predicates_to_native_binary() {
    let root = temp_dir("ptn-native-recursive-mkdir");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("recursive-mkdir.php");
    let output = root.join("recursive-mkdir-bin");
    fs::write(
        &input,
        "<?php\n\
$base = __DIR__ . \"/fs-tree\";\n\
$nested = $base . \"/one//two/three\";\n\
$file = $nested . \"/leaf.txt\";\n\
var_dump(file_exists($base));\n\
var_dump(mkdir($nested, 0777, true));\n\
var_dump(is_dir($base));\n\
var_dump(is_dir($base . \"/one/two/three\"));\n\
var_dump(file_exists($nested));\n\
var_dump(is_file($nested));\n\
var_dump(file_put_contents($file, \"x\"));\n\
var_dump(is_file($file));\n\
var_dump(file_exists($file));\n\
@unlink($file);\n\
var_dump(rmdir($base . \"/one/two/three\"));\n\
var_dump(rmdir($base . \"/one/two\"));\n\
var_dump(rmdir($base . \"/one\"));\n\
var_dump(rmdir($base));\n\
echo \"done\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(false)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(false)\n\
int(1)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
done\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_mkdir_existing_directory_and_nonrecursive_diagnostics_to_native_binary() {
    let root = temp_dir("ptn-native-mkdir-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("mkdir-diagnostics.php");
    let output = root.join("mkdir-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
$dir = __DIR__ . \"/exists\";\n\
$nested = $dir . \"/child/grand\";\n\
var_dump(mkdir($dir));\n\
var_dump(mkdir($dir));\n\
var_dump(mkdir($nested, 0777, false));\n\
var_dump(mkdir($nested, 0777, true));\n\
var_dump(mkdir($nested, 0777, true));\n\
var_dump(is_dir($nested));\n\
var_dump(rmdir($nested));\n\
var_dump(rmdir($dir . \"/child\"));\n\
var_dump(rmdir($dir));\n\
echo \"done\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let stdout = String::from_utf8(execution.stdout).unwrap();
    assert!(stdout.contains("bool(true)\n\nWarning: mkdir("));
    assert!(stdout.contains("): File exists in ptn on line 5\nbool(false)\n"));
    assert!(stdout.contains("): No such file or directory in ptn on line 6\nbool(false)\n"));
    assert!(stdout.contains("): File exists in ptn on line 8\nbool(false)\n"));
    assert!(stdout.ends_with(
        "bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
done\n"
    ));
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
fn compile_strrchr_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-strrchr-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strrchr.php");
    let output = root.join("strrchr-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(strrchr(\"\", \"\"));\n\
var_dump(strrchr(\"abc\", \"\"));\n\
var_dump(strrchr(\"\", \"abc\"));\n\
var_dump(strrchr(\"abc\", \"abc\"));\n\
var_dump(strrchr(\"test \".chr(0).\" test\", \" \"));\n\
var_dump(strrchr(\"test\".chr(0).\"string\", \"t\"));\n\
var_dump(strrchr(\"Hello, World\", \"World\", true));\n\
var_dump(strrchr(12345, 52));\n\
var_dump(function_exists(\"strrchr\"), function_exists(\"STRRCHR\"));\n\
?>",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(false)\nbool(false)\nbool(false)\nstring(3) \"abc\"\nstring(5) \" test\"\nstring(5) \"tring\"\nstring(7) \"Hello, \"\nstring(1) \"5\"\nbool(true)\nbool(true)\n"
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
fn compile_numeric_internals_reject_unsupported_operands_to_native_binary() {
    let root = temp_dir("ptn-native-numeric-internal-operand-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("numeric-internal-operand-diagnostics.php");
    let output = root.join("numeric-internal-operand-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
try { var_dump(abs([])); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(abs(new stdClass)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(abs(\"9x\")); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(sqrt([])); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(sqrt(new stdClass)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(sqrt(\"9x\")); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(fdiv([], 1)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(fdiv(1, [])); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(fdiv(\"9x\", 1)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(fdiv(1, new stdClass)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(abs(\"9\"));\n\
var_dump(abs(\"9.5\"));\n\
var_dump(sqrt(\"9\"));\n\
var_dump(sqrt(-1));\n\
var_dump(fdiv(\"9\", \"2\"));\n\
var_dump(fdiv(true, 2));\n\
var_dump(fdiv(0.0, 0.0));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "abs(): Argument #1 ($num) must be of type int|float, array given\n",
            "abs(): Argument #1 ($num) must be of type int|float, stdClass given\n",
            "abs(): Argument #1 ($num) must be of type int|float, string given\n",
            "sqrt(): Argument #1 ($num) must be of type float, array given\n",
            "sqrt(): Argument #1 ($num) must be of type float, stdClass given\n",
            "sqrt(): Argument #1 ($num) must be of type float, string given\n",
            "fdiv(): Argument #1 ($num1) must be of type float, array given\n",
            "fdiv(): Argument #2 ($num2) must be of type float, array given\n",
            "fdiv(): Argument #1 ($num1) must be of type float, string given\n",
            "fdiv(): Argument #2 ($num2) must be of type float, stdClass given\n",
            "int(9)\n",
            "float(9.5)\n",
            "float(3)\n",
            "float(NAN)\n",
            "float(4.5)\n",
            "float(0.5)\n",
            "float(NAN)\n",
        )
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
fn compile_intdiv_exception_edges_are_catchable_to_native_binary() {
    let root = temp_dir("ptn-native-intdiv-exception-edges");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("intdiv-exception-edges.php");
    let output = root.join("intdiv-exception-edges-bin");
    fs::write(
        &input,
        "<?php\n\
try {\n\
    var_dump(intdiv(PHP_INT_MIN, -1));\n\
} catch (\\ArithmeticError $e) {\n\
    echo get_class($e), ':', $e->getMessage(), \"\\n\";\n\
}\n\
try {\n\
    var_dump(intdiv(1, 0));\n\
} catch (\\ArithmeticError $e) {\n\
    echo get_class($e), ':', $e->getMessage(), \"\\n\";\n\
}\n\
echo \"after\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "ArithmeticError:Division of PHP_INT_MIN by -1 is not an integer\n\
DivisionByZeroError:Division by zero\n\
after\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_intdiv_unsupported_operands_are_type_errors_to_native_binary() {
    let root = temp_dir("ptn-native-intdiv-unsupported-operands");
    fs::create_dir_all(&root).unwrap();
    let data = root.join("payload.txt");
    fs::write(&data, "payload").unwrap();
    let input = root.join("intdiv-unsupported-operands.php");
    let output = root.join("intdiv-unsupported-operands-bin");
    let data_path = data.to_string_lossy();
    fs::write(
        &input,
        format!(
            "<?php\n\
try {{ var_dump(intdiv([], 2)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
try {{ var_dump(intdiv(6, new stdClass)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
$fp = fopen(\"{}\", \"r\");\n\
try {{ var_dump(intdiv($fp, 2)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
try {{ var_dump(intdiv(\"9x\", 2)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
try {{ var_dump(intdiv(6, [])); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
try {{ var_dump(intdiv(INF, 1)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
try {{ var_dump(intdiv(1, NAN)); }} catch (\\TypeError $e) {{ echo $e->getMessage(), \"\\n\"; }}\n\
echo \"after\\n\";",
            data_path
        ),
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "intdiv(): Argument #1 ($num1) must be of type int, array given\n\
intdiv(): Argument #2 ($num2) must be of type int, stdClass given\n\
intdiv(): Argument #1 ($num1) must be of type int, resource given\n\
intdiv(): Argument #1 ($num1) must be of type int, string given\n\
intdiv(): Argument #2 ($num2) must be of type int, array given\n\
intdiv(): Argument #1 ($num1) must be of type int, float given\n\
intdiv(): Argument #2 ($num2) must be of type int, float given\n\
after\n"
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
        "<?php var_dump(12300000.0); var_dump(24600000.0); var_dump(1.0e16); var_dump(1.0e17); var_dump(-9.22337203900226E+18); var_dump(1.4757395258967642E+19); var_dump(1.2e-5);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "float(12300000)\nfloat(24600000)\nfloat(10000000000000000)\nfloat(1.0E+17)\nfloat(-9.22337203900226E+18)\nfloat(1.4757395258967642E+19)\nfloat(1.2E-5)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn scalar_float_stringification_uses_php_exponent_spelling_in_native_binary() {
    let root = temp_dir("ptn-native-scalar-float-exponent-spelling");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-float-exponent-spelling.php");
    let output = root.join("scalar-float-exponent-spelling-bin");
    fs::write(
        &input,
        "<?php echo 9.2233720368548E+18, \"\\n\", 1.2e-5, \"\\n\", 1.0e17, \"\\n\"; echo INF, \" \", -INF, \" \", NAN, \"\\n\"; echo strlen(1.2e-5), \" \", strlen(1.0e17), \" \", strlen(INF), \" \", strlen(NAN), \"\\n\"; echo \"x\" . 1.0e17, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "9.2233720368548E+18\n1.2E-5\n1.0E+17\nINF -INF NAN\n6 7 3 3\nx1.0E+17\n"
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
        "<?php\n\
var_dump(chunk_split(12345, 2, \".\"), chunk_split(\"abc\", \"2\", true), function_exists(\"chunk_split\"), function_exists(\"CHUNK_SPLIT\"));\n\
try { chunk_split(\"abc\", 0); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { chunk_split(\"abc\", -1); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(8) \"12.34.5.\"\nstring(5) \"ab1c1\"\nbool(true)\nbool(true)\nchunk_split(): Argument #2 ($length) must be greater than 0\nchunk_split(): Argument #2 ($length) must be greater than 0\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_str_repeat_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-str-repeat-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("str-repeat-function.php");
    let output = root.join("str-repeat-function-bin");
    fs::write(
        &input,
        "<?php var_dump(str_repeat(\"ab\", 3), str_repeat(\"x\", 0), str_repeat(7, \"2\"), function_exists(\"str_repeat\"), function_exists(\"STR_REPEAT\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(6) \"ababab\"\nstring(0) \"\"\nstring(2) \"77\"\nbool(true)\nbool(true)\n"
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
fn compile_addcslashes_stripcslashes_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-cslashes-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("cslashes-phpt-shape.php");
    let output = root.join("cslashes-phpt-shape-bin");
    fs::write(
        &input,
        r#"<?php
echo addcslashes("", "")."\n";
echo addcslashes("", "burp")."\n";
echo addcslashes("kaboemkara!", "")."\n";
echo addcslashes("foobarbaz", 'bar')."\n";
echo addcslashes('foo[ ]', 'A..z')."\n";
echo @addcslashes("zoo['.']", 'z..A')."\n";
echo addcslashes('abcdefghijklmnopqrstuvwxyz', "a\145..\160z")."\n";
echo "\n\r" == stripcslashes('\n\r'),"\n";
echo stripcslashes('\065\x64')."\n";
echo stripcslashes('')."\n";
var_dump(function_exists("addcslashes"), function_exists("ADDCSlASHES"), function_exists("stripcslashes"), function_exists("STRIPCSLASHES"));
?>"#,
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        r#"

kaboemkara!
foo\b\a\r\b\az
\f\o\o\[ \]
\zoo['\.']
\abcd\e\f\g\h\i\j\k\l\m\n\o\pqrstuvwxy\z
1
5d

bool(true)
bool(true)
bool(true)
bool(true)
"#
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    for function in ["ptn_internal_addcslashes", "ptn_internal_stripcslashes"] {
        let marker = format!("static PtnValue {function}(");
        let body = generated_c_static_function_body(&c_source, &marker);
        assert!(
            body.contains("ptn_value_to_string_operand")
                || body.contains("ptn_internal_expect_string_arg"),
            "{function} should use the direct string operand helper"
        );
        assert!(
            !body.contains("ptn_value_to_string(args"),
            "{function} should not convert direct argument expressions unconditionally"
        );
    }
}

#[test]
fn compile_addslashes_basic_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-addslashes-basic-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("addslashes-basic.php");
    let output = root.join("addslashes-basic-bin");
    fs::write(
        &input,
        "<?php\n\
echo \"*** Testing addslashes() : basic functionality ***\\n\";\n\
$str_array = array(\n\
    \"How's everybody\",\n\
    'Are you \"JOHN\"?',\n\
    'c:\\php\\addslashes',\n\
    \"hello\\0world\"\n\
);\n\
foreach ($str_array as $str) {\n\
    var_dump(addslashes($str));\n\
}\n\
echo \"Done\\n\";\n\
var_dump(function_exists(\"addslashes\"), function_exists(\"ADDSLASHES\"));\n\
?>",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"*** Testing addslashes() : basic functionality ***\n\
string(16) \"How\\'s everybody\"\n\
string(17) \"Are you \\\"JOHN\\\"?\"\n\
string(19) \"c:\\\\php\\\\addslashes\"\n\
string(12) \"hello\\0world\"\n\
Done\n\
bool(true)\n\
bool(true)\n"
            .to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let body =
        generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_addslashes(");
    assert!(
        body.contains("ptn_value_to_string_operand")
            || body.contains("ptn_internal_expect_string_arg"),
        "addslashes should use the direct string operand helper"
    );
    assert!(
        !body.contains("ptn_value_to_string(args"),
        "addslashes should not convert direct argument expressions unconditionally"
    );
}

#[test]
fn compile_addslashes_stripslashes_round_trip_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-add-stripslashes-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("add-stripslashes.php");
    let output = root.join("add-stripslashes-bin");
    fs::write(
        &input,
        "<?php\n\
$input = '';\n\
for ($i = 0; $i < 512; $i++) {\n\
    $input .= chr($i % 256);\n\
}\n\
echo \"Normal: \";\n\
if ($input === stripslashes(addslashes($input))) {\n\
    echo \"OK\\n\";\n\
} else {\n\
    echo \"FAILED\\n\";\n\
}\n\
var_dump(stripslashes(\"A\\\\0B\\\\nC\\\\\\\\\"), function_exists(\"stripslashes\"), function_exists(\"STRIPSLASHES\"));\n\
?>",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"Normal: OK\nstring(6) \"A\0BnC\\\"\nbool(true)\nbool(true)\n".to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let body =
        generated_c_static_function_body(&c_source, "static PtnValue ptn_internal_stripslashes(");
    assert!(
        body.contains("ptn_value_to_string_operand")
            || body.contains("ptn_internal_expect_string_arg"),
        "stripslashes should use the direct string operand helper"
    );
    assert!(
        !body.contains("ptn_value_to_string(args"),
        "stripslashes should not convert direct argument expressions unconditionally"
    );
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
fn compile_basename_binary_safe_suffix_to_native_binary() {
    let root = temp_dir("ptn-native-basename-binary-safe-suffix");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("basename.php");
    let output = root.join("basename-bin");
    fs::write(
        &input,
        r#"<?php
var_dump(basename("bar"));
var_dump(basename("/foo/bar"));
var_dump(basename("/foo/bar/"));
var_dump(basename("bar.gz", ".gz"));
var_dump(basename("/.gz", ".gz"));
var_dump(basename("bar.gz", "bar.gz"));
var_dump(basename("foo\\", "\\"));
echo bin2hex(basename("foo".chr(0)."bar.gz", ".gz")), "\n";
var_dump(basename(12345));
var_dump(function_exists("basename"), function_exists("BASENAME"));
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        execution.stdout,
        b"string(3) \"bar\"\n\
string(3) \"bar\"\n\
string(3) \"bar\"\n\
string(3) \"bar\"\n\
string(3) \".gz\"\n\
string(6) \"bar.gz\"\n\
string(3) \"foo\"\n\
666f6f00626172\n\
string(5) \"12345\"\n\
bool(true)\n\
bool(true)\n"
            .to_vec()
    );
    assert_eq!(execution.stderr, Vec::<u8>::new());
}

#[test]
fn compile_pathinfo_components_and_flags_to_native_binary() {
    let root = temp_dir("ptn-native-pathinfo-components-and-flags");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("pathinfo.php");
    let output = root.join("pathinfo-bin");
    fs::write(
        &input,
        r#"<?php
var_dump(PATHINFO_DIRNAME, PATHINFO_BASENAME, PATHINFO_EXTENSION, PATHINFO_FILENAME, PATHINFO_ALL);
var_dump(function_exists("pathinfo"), defined("PATHINFO_ALL"), constant("PATHINFO_FILENAME"));
var_dump(pathinfo(""));
var_dump(pathinfo("."));
var_dump(pathinfo("/foo/bar.txt"));
var_dump(pathinfo("/foo/bar", PATHINFO_EXTENSION));
var_dump(pathinfo("/foo/bar.", PATHINFO_EXTENSION));
echo bin2hex(pathinfo("foo" . chr(0) . "bar.txt", PATHINFO_FILENAME)), "\n";
try {
    pathinfo("x", PATHINFO_EXTENSION | PATHINFO_FILENAME);
} catch (\ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    pathinfo("x", PATHINFO_ALL + 1);
} catch (\ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    pathinfo([]);
} catch (\TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "int(1)\n",
            "int(2)\n",
            "int(4)\n",
            "int(8)\n",
            "int(15)\n",
            "bool(true)\n",
            "bool(true)\n",
            "int(8)\n",
            "array(2) {\n",
            "  [\"basename\"]=>\n",
            "  string(0) \"\"\n",
            "  [\"filename\"]=>\n",
            "  string(0) \"\"\n",
            "}\n",
            "array(4) {\n",
            "  [\"dirname\"]=>\n",
            "  string(1) \".\"\n",
            "  [\"basename\"]=>\n",
            "  string(1) \".\"\n",
            "  [\"extension\"]=>\n",
            "  string(0) \"\"\n",
            "  [\"filename\"]=>\n",
            "  string(0) \"\"\n",
            "}\n",
            "array(4) {\n",
            "  [\"dirname\"]=>\n",
            "  string(4) \"/foo\"\n",
            "  [\"basename\"]=>\n",
            "  string(7) \"bar.txt\"\n",
            "  [\"extension\"]=>\n",
            "  string(3) \"txt\"\n",
            "  [\"filename\"]=>\n",
            "  string(3) \"bar\"\n",
            "}\n",
            "string(0) \"\"\n",
            "string(0) \"\"\n",
            "666f6f00626172\n",
            "pathinfo(): Argument #2 ($flags) must be only one of the PATHINFO_* constants\n",
            "pathinfo(): Argument #2 ($flags) must be one of the PATHINFO_* constants\n",
            "pathinfo(): Argument #1 ($path) must be of type string, array given\n",
        )
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}0\n",
            undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn parser_accepts_named_arguments_on_calls() {
    let program = parser::parse(
        "<?php function pair($left, $right) { return $left . $right; } pair(right: 2, left: 1);",
    )
    .unwrap();
    let Statement::Call { argument_names, .. } = &program.statements[0] else {
        panic!("expected call statement");
    };
    assert_eq!(
        argument_names,
        &vec![Some("right".to_string()), Some("left".to_string())]
    );
}

#[test]
fn compile_user_function_named_arguments_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-named-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-named-arguments.php");
    let output = root.join("user-function-named-arguments-bin");
    fs::write(
        &input,
        "<?php
function side($label) { echo \"arg:$label\\n\"; return $label; }
function collect($left, $middle, $right) {
    return $left . \"|\" . $middle . \"|\" . $right . \"|\" . func_num_args();
}
echo collect(right: side(\"R\"), left: side(\"L\"), middle: side(\"M\")), \"\\n\";
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "arg:R\narg:L\narg:M\nL|M|R|3\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_named_argument_diagnostics_to_native_binary() {
    let cases = [
        (
            "unknown",
            "<?php function target($first) { return $first; } target(missing: 1);",
            "Fatal error: Unknown named parameter $missing\n",
        ),
        (
            "duplicate",
            "<?php function target($first, $second) { return $first; } target(\"positional\", first: \"named\");",
            "Fatal error: Named parameter $first overwrites previous argument\n",
        ),
    ];

    for (name, source, expected_stderr) in cases {
        let root_name = format!("ptn-native-user-function-named-argument-{name}");
        let root = temp_dir(&root_name);
        fs::create_dir_all(&root).unwrap();
        let input = root.join(format!("{name}.php"));
        let output = root.join(format!("{name}-bin"));
        fs::write(&input, source).unwrap();

        compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

        let execution = Command::new(&output).output().unwrap();
        assert!(!execution.status.success());
        assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
        assert_eq!(
            String::from_utf8(execution.stderr).unwrap(),
            expected_stderr
        );
    }
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
fn compile_user_function_default_arguments_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-default-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-default-arguments.php");
    let output = root.join("user-function-default-arguments-bin");
    fs::write(
        &input,
        "<?php\n\
function inspect($count = -10, $label = \"seed\", $flag = false, $none = null) {\n\
    var_dump($count, $label, $flag, $none, func_num_args(), func_get_args());\n\
    $label = \"changed\";\n\
}\n\
inspect();\n\
inspect(20, \"passed\");\n\
inspect();",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(-10)\nstring(4) \"seed\"\nbool(false)\nNULL\nint(0)\narray(0) {\n}\nint(20)\nstring(6) \"passed\"\nbool(false)\nNULL\nint(2)\narray(2) {\n  [0]=>\n  int(20)\n  [1]=>\n  string(6) \"passed\"\n}\nint(-10)\nstring(4) \"seed\"\nbool(false)\nNULL\nint(0)\narray(0) {\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_array_default_arguments_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-array-default-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-array-default-arguments.php");
    let output = root.join("user-function-array-default-arguments-bin");
    fs::write(
        &input,
        "<?php\n\
function inspect($items = [\"seed\" => \"value\", 4 => \"four\"], $empty = array()) {\n\
    var_dump($items, $empty);\n\
    $items[\"seed\"] = \"changed\";\n\
    $empty[] = \"local\";\n\
}\n\
inspect();\n\
inspect();",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [\"seed\"]=>\n  string(5) \"value\"\n  [4]=>\n  string(4) \"four\"\n}\narray(0) {\n}\narray(2) {\n  [\"seed\"]=>\n  string(5) \"value\"\n  [4]=>\n  string(4) \"four\"\n}\narray(0) {\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_user_function_default_argument_too_few_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-default-argument-too-few");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-default-argument-too-few.php");
    let output = root.join("user-function-default-argument-too-few-bin");
    fs::write(
        &input,
        "<?php function required_then_optional($required, $optional = 2) { return $required; } required_then_optional();",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: required_then_optional() expects at least 1 argument, 0 given\n"
    );
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
fn compile_variadic_user_function_arguments_to_native_binary() {
    let root = temp_dir("ptn-native-variadic-user-function-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("variadic-user-function-arguments.php");
    let output = root.join("variadic-user-function-arguments-bin");
    fs::write(
        &input,
        "<?php function inspect($head, ...$tail) { $tail[0] = \"changed\"; var_dump($head, $tail, func_num_args(), func_get_arg(1), func_get_args()); } inspect(\"h\", \"a\", \"b\");",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"h\"\narray(2) {\n  [0]=>\n  string(7) \"changed\"\n  [1]=>\n  string(1) \"b\"\n}\nint(3)\nstring(1) \"a\"\narray(3) {\n  [0]=>\n  string(1) \"h\"\n  [1]=>\n  string(1) \"a\"\n  [2]=>\n  string(1) \"b\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source
        .contains("ptn_runtime_set_call_frame(&runtime, argc, args, 1, ptn_parameter_names);"));
    assert!(c_source.contains("ptn_array_set_entry(ptn_variadic_1.as.array"));
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
fn compile_func_get_arg_bounds_errors_to_native_binary() {
    let root = temp_dir("ptn-native-func-get-arg-bounds-errors");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("func-get-arg-bounds-errors.php");
    let output = root.join("func-get-arg-bounds-errors-bin");
    fs::write(
        &input,
        "<?php function inspect($value) { try { func_get_arg(-1); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; } try { func_get_arg(1); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; } } inspect(\"one\");",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0\nfunc_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function\n"
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
    assert!(c_source.contains("ptn_user_function_0(&runtime, ptn_null(), 1,"));

    let main_start = c_source.find("\nint main(void)").unwrap();
    let main_body = &c_source[main_start..];
    assert!(main_body.contains("ptn_user_function_1(&runtime, ptn_null(), 1,"));
    assert!(!main_body.contains("ptn_call_function(&runtime, \"apply\""));
    assert!(!c_source.contains("ptn_call_internal"));
    assert!(!c_source.contains("ptn_internal_var_dump"));
}

#[test]
fn compile_dynamic_function_value_calls_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-function-value-call");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-function-value-call.php");
    let output = root.join("dynamic-function-value-call-bin");
    fs::write(
        &input,
        "<?php\n\
function add_one($value) { return $value + 1; }\n\
function push_marker(&$items) { $items[] = \"mark\"; return count($items); }\n\
$call = \"add_one\";\n\
echo $call(6), \"\\n\";\n\
$call = \"push_marker\";\n\
$items = [\"seed\"];\n\
echo $call($items), \":\", count($items), \":\", $items[1], \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "7\n2:2:mark\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_dynamic_function_name("));
    assert!(c_source.contains("ptn_call_callable(runtime"));
    assert!(c_source.contains("ptn_call_dynamic_function_name(runtime"));
    assert!(c_source.contains("ptn_runtime_reference_for_variable(&runtime, \"items\")"));
    assert!(c_source.contains("ptn_dynamic_call_detach_first_reference_argument"));
}

#[test]
fn compile_static_method_callable_value_call_to_native_binary() {
    let root = temp_dir("ptn-native-static-method-callable-value-call");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("static-method-callable-value-call.php");
    let output = root.join("static-method-callable-value-call-bin");
    fs::write(
        &input,
        "<?php
class MathBox {
    public static function pair($left, $right) {
        return $left * 10 + $right;
    }
}

$call = [\"MathBox\", \"pair\"];
echo $call(3, 4), \"\\n\";
$keyed = [1 => \"pair\", 0 => \"MathBox\"];
echo $keyed(5, 6), \"\\n\";
echo MathBox::pair(1, 2), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "34\n56\n12\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_callable_function_name("));
    assert!(c_source.contains("ptn_call_callable(runtime"));
    assert!(c_source.contains("MathBox::pair"));
}

#[test]
fn compile_instance_method_call_user_func_to_native_binary() {
    let root = temp_dir("ptn-native-instance-method-call-user-func");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("instance-method-call-user-func.php");
    let output = root.join("instance-method-call-user-func-bin");
    fs::write(
        &input,
        "<?php
class Greeter {
    public function label($value) {
        return \"item=\" . $value;
    }

    public function via_this($value) {
        return $this->label($value + 1);
    }
}

$greeter = new Greeter();
echo $greeter->label(3), \"\\n\";
echo call_user_func([$greeter, \"label\"], 4), \"\\n\";
echo call_user_func([$greeter, \"via_this\"], 5), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "item=3\nitem=4\nitem=6\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_new_shell(&runtime, \"Greeter\")"));
    assert!(c_source.contains("ptn_call_declared_method(&runtime"));
    assert!(c_source.contains("ptn_call_callable(runtime"));
    assert!(c_source.contains("ptn_runtime_write_variable(&runtime, \"this\", receiver);"));
}

#[test]
fn compile_magic_call_object_callables_to_native_binary() {
    let root = temp_dir("ptn-native-magic-call-object-callables");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("magic-call-object-callables.php");
    let output = root.join("magic-call-object-callables-bin");
    fs::write(
        &input,
        "<?php
class MagicBase {
    public function __call($name, $args) {
        return $name . \":\" . count($args) . \":\" . $args[0];
    }
}

class MagicChild extends MagicBase {
}

$child = new MagicChild();
echo $child->direct(\"one\"), \"\\n\";
echo call_user_func([$child, \"via_user\"], \"two\"), \"\\n\";
$dynamic = [$child, \"via_dynamic\"];
echo $dynamic(\"three\"), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "direct:1:one\nvia_user:1:two\nvia_dynamic:1:three\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_declared_class_has_call_magic"));
    assert!(c_source.contains("ptn_magic_args[0] = ptn_string(method_name);"));
    assert!(c_source.contains("MagicBase::__call"));
    assert!(c_source.contains("ptn_call_callable(&runtime"));
}

#[test]
fn compile_is_callable_object_callable_subset_to_native_binary() {
    let root = temp_dir("ptn-native-is-callable-object-subset");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("is-callable-object-subset.php");
    let output = root.join("is-callable-object-subset-bin");
    fs::write(
        &input,
        "<?php
class CallableBase {
    public function inherited() {
    }
}

class CallableWorker extends CallableBase {
    public static function stat() {
    }

    public function own() {
    }
}

class CallableMagic extends CallableBase {
    public function __call($name, $args) {
    }
}

$worker = new CallableWorker();
$magic = new CallableMagic();
$closure = function () { return 1; };
var_dump(is_callable(\"strlen\"));
var_dump(is_callable(\"missing_function\"));
var_dump(is_callable(\"missing_function\", true));
var_dump(is_callable($closure));
var_dump(is_callable([$worker, \"inherited\"]));
var_dump(is_callable([$worker, \"missing\"]));
var_dump(is_callable([$magic, \"missing\"]));
var_dump(is_callable([\"CallableWorker\", \"stat\"]));
var_dump(is_callable([\"CallableWorker\", \"own\"]));
var_dump(is_callable([\"CallableWorker\", \"missing\"], true));
var_dump(is_callable([1, \"x\"], true));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_is_callable"));
    assert!(c_source.contains("ptn_callable_is_valid"));
    assert!(c_source.contains("ptn_declared_class_static_method_exists"));
    assert!(c_source.contains("ptn_declared_class_has_call_magic"));
}

#[test]
fn compile_declared_public_instance_properties_to_native_binary() {
    let root = temp_dir("ptn-native-declared-public-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("declared-public-properties.php");
    let output = root.join("declared-public-properties-bin");
    fs::write(
        &input,
        "<?php
class Box {
    public $name = \"ptn\";
    public $count = 2, $unset;
}

$box = new Box;
var_dump($box->name);
var_dump($box->count);
var_dump($box->unset);
$box->name = \"native\";
var_dump($box->name);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(3) \"ptn\"\nint(2)\nNULL\nstring(6) \"native\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_new_shell(&runtime, \"Box\")"));
    assert!(c_source.contains("ptn_object_write_property(&runtime"));
    assert!(c_source.contains("\"name\""));
    assert!(c_source.contains("\"count\""));
    assert!(c_source.contains("\"unset\""));
}

#[test]
fn compile_private_instance_properties_to_native_binary() {
    let root = temp_dir("ptn-native-private-instance-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("private-instance-properties.php");
    let output = root.join("private-instance-properties-bin");
    fs::write(
        &input,
        "<?php
class SecretBox {
    private $secret = \"seed\";
    public $label = \"pub\";

    public function __construct($secret) {
        $this->secret = $secret;
    }

    public function reveal() {
        return $this->secret;
    }

    public function replace($value) {
        $this->secret = $value;
        return $this->secret;
    }
}

$box = new SecretBox(\"kept\");
var_dump($box->reveal());
var_dump($box->replace(\"next\"));
var_dump($box);
try {
    var_dump($box->secret);
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
try {
    $box->secret = \"bad\";
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
var_dump($box->reveal());
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(4) \"kept\"\n",
            "string(4) \"next\"\n",
            "object(SecretBox)#1 (2) {\n",
            "  [\"secret\":\"SecretBox\":private]=>\n",
            "  string(4) \"next\"\n",
            "  [\"label\"]=>\n",
            "  string(3) \"pub\"\n",
            "}\n",
            "Cannot access private property SecretBox::$secret\n",
            "Cannot access private property SecretBox::$secret\n",
            "string(4) \"next\"\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_declared_private_property_class"));
    assert!(c_source.contains("\"SecretBox\""));
    assert!(c_source.contains("\"secret\""));
}

#[test]
fn compile_declared_non_public_instance_properties_to_native_binary() {
    let root = temp_dir("ptn-native-declared-non-public-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("declared-non-public-properties.php");
    let output = root.join("declared-non-public-properties-bin");
    fs::write(
        &input,
        "<?php
class Box {
    private $secret = 1;
    protected $guarded = 2;

    public function __construct($extra) {
        $this->secret = $this->secret + $extra;
        $this->guarded = $this->guarded + $extra;
    }

    public function total() {
        return $this->secret + $this->guarded;
    }
}

$box = new Box(3);
echo $box->total(), \"\\n\";
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "9\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_declared_class_constructor_to_native_binary() {
    let root = temp_dir("ptn-native-declared-class-constructor");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("declared-class-constructor.php");
    let output = root.join("declared-class-constructor-bin");
    fs::write(
        &input,
        "<?php
class Box {
    public static $made = 0;
    public $name = \"unset\";
    public $count = 0;

    public function __construct($name, $count = 1) {
        $this->name = $name;
        $this->count = $count;
        self::$made = self::$made + 1;
    }
}

$box = new Box(\"native\", 7);
echo $box->name, \":\", $box->count, \":\", Box::$made, \"\\n\";
$default = new Box(\"default\");
echo $default->name, \":\", $default->count, \":\", Box::$made, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "native:7:1\ndefault:1:2\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_new_shell(&runtime, \"Box\")"));
    assert!(c_source.contains("ptn_call_declared_method(&runtime"));
    assert!(c_source.contains("\"__construct\""));
}

#[test]
fn compile_inherited_declared_class_constructor_to_native_binary() {
    let root = temp_dir("ptn-native-inherited-declared-class-constructor");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inherited-declared-class-constructor.php");
    let output = root.join("inherited-declared-class-constructor-bin");
    fs::write(
        &input,
        "<?php
class BaseBox {
    public function __construct($name) {
        $this->name = \"base:\" . $name;
    }
}

class ChildBox extends BaseBox {
    public function label() {
        return $this->name;
    }
}

$child = new ChildBox(\"inherited\");
echo $child->label(), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "base:inherited\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_new_shell(&runtime, \"ChildBox\")"));
    assert!(c_source.contains("BaseBox::__construct"));
    assert!(c_source.contains("ptn_call_declared_method(&runtime"));
}

#[test]
fn compile_declared_class_destructor_runs_at_shutdown_to_native_binary() {
    let root = temp_dir("ptn-native-declared-class-destructor-shutdown");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("declared-class-destructor-shutdown.php");
    let output = root.join("declared-class-destructor-shutdown-bin");
    fs::write(
        &input,
        "<?php
class Test {
    public function __construct() {
        echo __METHOD__, \"\\n\";
    }

    public function __destruct() {
        echo __METHOD__, \"\\n\";
    }
}

$object = new Test;
echo \"===DONE===\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Test::__construct\n===DONE===\nTest::__destruct\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_run_object_destructors(runtime)"));
    assert!(c_source.contains("\"__destruct\""));
}

#[test]
fn compile_inherited_class_destructor_runs_on_unset_to_native_binary() {
    let root = temp_dir("ptn-native-inherited-class-destructor-unset");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inherited-class-destructor-unset.php");
    let output = root.join("inherited-class-destructor-unset-bin");
    fs::write(
        &input,
        "<?php
class base {
    public function __construct() {
        echo __METHOD__, \"\\n\";
    }

    public function __destruct() {
        echo __METHOD__, \"\\n\";
    }
}

class derived extends base {
}

$object = new derived;
unset($object);
echo \"Done\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "base::__construct\nbase::__destruct\nDone"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_declared_class_method_exists"));
    assert!(c_source.contains("base::__destruct"));
}

#[test]
fn compile_declared_class_metadata_intrinsics_to_native_binary() {
    let root = temp_dir("ptn-native-declared-class-metadata-intrinsics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("declared-class-metadata-intrinsics.php");
    let output = root.join("declared-class-metadata-intrinsics-bin");
    fs::write(
        &input,
        "<?php
class Worker {
    public function Run($value) {
        return $value;
    }

    public static function StaticWork() {
    }
}

$worker = new Worker();
$callback = function () { return null; };
var_dump(class_exists(\"Worker\"));
var_dump(class_exists(\"worker\"));
var_dump(class_exists(\"Missing\", false));
var_dump(class_exists(\"stdClass\"));
var_dump(method_exists(\"Worker\", \"run\"));
var_dump(method_exists($worker, \"RUN\"));
var_dump(method_exists(\"Worker\", \"staticwork\"));
var_dump(method_exists(\"Worker\", \"missing\"));
var_dump(method_exists(\"stdClass\", \"anything\"));
var_dump(get_class($worker));
var_dump(get_class(new stdClass));
var_dump(get_class($callback));
try {
    get_class(42);
} catch (TypeError $e) {
    echo get_class($e), \": \", $e->getMessage(), \"\\n\";
}
var_dump(function_exists(\"get_class\"));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(6) \"Worker\"\n",
            "string(8) \"stdClass\"\n",
            "string(7) \"Closure\"\n",
            "TypeError: get_class(): Argument #1 ($object) must be of type object, int given\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static int ptn_declared_class_exists("));
    assert!(c_source.contains("static int ptn_declared_class_method_exists("));
    assert!(c_source.contains("ptn_internal_class_exists"));
    assert!(c_source.contains("ptn_internal_get_class"));
    assert!(c_source.contains("ptn_internal_method_exists"));
}

#[test]
fn compile_reflection_function_metadata_to_native_binary() {
    let root = temp_dir("ptn-native-reflection-function-metadata");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("reflection-function-metadata.php");
    let output = root.join("reflection-function-metadata-bin");
    fs::write(
        &input,
        "<?php
namespace A\\B;

function foo($first, $second = null, ...$rest) {
}

$internal = new \\ReflectionFunction(\"sort\");
var_dump($internal->getName());
var_dump($internal->isInternal());
var_dump($internal->isUserDefined());
var_dump($internal->inNamespace());
var_dump($internal->getNamespaceName());
var_dump($internal->getShortName());
var_dump($internal->isVariadic());
var_dump($internal->getNumberOfParameters());
var_dump($internal->getNumberOfRequiredParameters());

$user = new \\ReflectionFunction(\"A\\\\B\\\\foo\");
var_dump($user->getName());
var_dump($user->isInternal());
var_dump($user->isUserDefined());
var_dump($user->inNamespace());
var_dump($user->getNamespaceName());
var_dump($user->getShortName());
var_dump($user->isVariadic());
var_dump($user->getNumberOfParameters());
var_dump($user->getNumberOfRequiredParameters());
var_dump(\\class_exists(\"ReflectionFunction\"));
var_dump(\\method_exists(\"ReflectionFunction\", \"getName\"));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(4) \"sort\"\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(0) \"\"\n",
            "string(4) \"sort\"\n",
            "bool(false)\n",
            "int(2)\n",
            "int(1)\n",
            "string(7) \"A\\B\\foo\"\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "string(3) \"A\\B\"\n",
            "string(3) \"foo\"\n",
            "bool(true)\n",
            "int(3)\n",
            "int(1)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_user_function_metadata"));
    assert!(c_source.contains("ptn_reflection_function_new"));
    assert!(c_source.contains("ptn_reflection_function_call_method"));
}

#[test]
fn compile_inherited_public_instance_methods_to_native_binary() {
    let root = temp_dir("ptn-native-inherited-public-instance-methods");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inherited-public-instance-methods.php");
    let output = root.join("inherited-public-instance-methods-bin");
    fs::write(
        &input,
        "<?php
class BaseLabeler {
    public function label($value) {
        return \"base=\" . $value;
    }

    public function same() {
        return \"base\";
    }
}

class ChildLabeler extends BaseLabeler {
    public function own($value) {
        return $this->label($value + 1);
    }

    public function same() {
        return \"child\";
    }
}

$child = new ChildLabeler();
echo $child->label(3), \"\\n\";
echo call_user_func([$child, \"label\"], 4), \"\\n\";
echo $child->own(5), \"\\n\";
echo $child->same(), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "base=3\nbase=4\nbase=6\nchild\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_new_shell(&runtime, \"ChildLabeler\")"));
    assert!(c_source.contains("ptn_call_declared_method(&runtime"));
    assert!(c_source.contains("BaseLabeler::label"));
    assert!(c_source.contains("ChildLabeler::same"));
}

#[test]
fn compile_inherited_private_and_child_public_properties_to_native_binary() {
    let root = temp_dir("ptn-native-inherited-private-child-public-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inherited-private-child-public-properties.php");
    let output = root.join("inherited-private-child-public-properties-bin");
    fs::write(
        &input,
        "<?php
class A {
    private $p = \"A::p\";

    public function showA() {
        echo $this->p, \"\\n\";
    }
}

class B extends A {
    public $p = \"B::p\";

    public function showB() {
        echo $this->p, \"\\n\";
    }
}

$b = new B;
$b->showA();
$b->showB();
echo $b->p, \"\\n\";
var_dump($b);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "A::p\n",
            "B::p\n",
            "B::p\n",
            "object(B)#1 (2) {\n",
            "  [\"p\":\"A\":private]=>\n",
            "  string(4) \"A::p\"\n",
            "  [\"p\"]=>\n",
            "  string(4) \"B::p\"\n",
            "}\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_declare_property(&runtime"));
    assert!(c_source.contains("PTN_PROPERTY_PRIVATE"));
    assert!(c_source.contains("PTN_PROPERTY_PUBLIC"));
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
fn compile_void_typed_user_function_to_native_binary() {
    let root = temp_dir("ptn-native-user-function-void-type");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("user-function-void-type.php");
    let output = root.join("user-function-void-type-bin");
    fs::write(
        &input,
        "<?php\n\
function explicit_done($value): void { echo \"value=$value\\n\"; return; }\n\
function implicit_done(): void { echo \"implicit\\n\"; }\n\
explicit_done(7);\n\
var_dump(explicit_done(8));\n\
var_dump(implicit_done());",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "value=7\nvalue=8\nNULL\nimplicit\nNULL\n"
    );
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
fn compile_array_object_type_predicates_to_native_binary() {
    let root = temp_dir("ptn-native-array-object-type-predicates");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-object-type-predicates.php");
    let output = root.join("array-object-type-predicates-bin");
    fs::write(
        &input,
        "<?php
class Box {}

$items = [1, \"two\" => 2];
$empty = [];
$std = new stdClass;
$box = new Box();
$callback = function () { return 1; };

var_dump(gettype($items), is_array($items), is_object($items));
var_dump(gettype($empty), is_array($empty), is_object($empty));
var_dump(gettype($std), is_array($std), is_object($std));
var_dump(gettype($box), is_array($box), is_object($box));
var_dump(gettype($callback), is_array($callback), is_object($callback));
var_dump(is_array(null), is_object(null), is_array(42), is_object(42), is_array(\"x\"), is_object(\"x\"), is_array(true), is_object(true));
var_dump(function_exists(\"is_array\"), function_exists(\"IS_OBJECT\"), function_exists(\"is_resource\"));
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"array\"\nbool(true)\nbool(false)\nstring(5) \"array\"\nbool(true)\nbool(false)\nstring(6) \"object\"\nbool(false)\nbool(true)\nstring(6) \"object\"\nbool(false)\nbool(true)\nstring(6) \"object\"\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_is_countable_current_value_subset_to_native_binary() {
    let root = temp_dir("ptn-native-is-countable-current-subset");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("is-countable-current-subset.php");
    let output = root.join("is-countable-current-subset-bin");
    fs::write(
        &input,
        "<?php
class Box {}

$items = [1, 2, 3];
$alias =& $items;
$nested = [[\"x\" => 1]];
$box = new Box();
$std = new stdClass;
$callback = function () { return 1; };

var_dump(is_countable($items));
var_dump(is_countable($alias));
var_dump(is_countable($nested[0]));
var_dump(is_countable($box));
var_dump(is_countable($std));
var_dump(is_countable($callback));
var_dump(is_countable(null), is_countable(false), is_countable(1), is_countable(\"x\"));
if (is_countable($items)) {
    var_dump(count($items));
}
var_dump(function_exists(\"is_countable\"), function_exists(\"IS_COUNTABLE\"));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "int(3)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_is_countable"));
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
        "<?php echo chr(72). chr(101) . chr(108) . chr(108). chr(111); echo chr(10); echo bin2hex(chr(255)), \" \", bin2hex(chr(254)), \" \", bin2hex(chr(\"65\")), \"\\n\"; var_dump(function_exists(\"chr\"), function_exists(\"CHR\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Hello\nff fe 41\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_out_of_range_phpt_shape_to_native_binary() {
    let root = temp_dir("ptn-native-chr-out-of-range-phpt-shape");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-out-of-range.php");
    let output = root.join("chr-out-of-range-bin");
    fs::write(
        &input,
        "<?php\n\nvar_dump(\"\\xFF\" == chr(-1));\nvar_dump(\"\\0\" == chr(256));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256 in ptn on line 3\nbool(true)\n\nDeprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256 in ptn on line 4\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_out_of_range_deprecation_suppression_to_native_binary() {
    let root = temp_dir("ptn-native-chr-out-of-range-deprecation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-out-of-range-deprecation.php");
    let output = root.join("chr-out-of-range-deprecation-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(\"\\xFF\" == chr(-1));\n\
var_dump(\"\\0\" == chr(256));\n\
error_reporting(E_ERROR);\n\
var_dump(bin2hex(chr(-2)));\n\
error_reporting(E_ALL);\n\
var_dump(bin2hex(@chr(257)));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256 in ptn on line 2\n\
bool(true)\n\
\n\
Deprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256 in ptn on line 3\n\
bool(true)\n\
string(2) \"fe\"\n\
string(2) \"01\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_precision_deprecations_use_call_site_to_native_binary() {
    let root = temp_dir("ptn-native-chr-precision-deprecations");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-precision-deprecations.php");
    let output = root.join("chr-precision-deprecations-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(bin2hex(chr(65.7)));\n\
var_dump(bin2hex(chr(\"65.7\")));\n\
error_reporting(E_ERROR);\n\
var_dump(bin2hex(chr(66.8)));\n\
error_reporting(E_ALL);\n\
var_dump(bin2hex(@chr(\"67.9\")));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Implicit conversion from float 65.7 to int loses precision in ptn on line 2\n\
string(2) \"41\"\n\
\n\
Deprecated: Implicit conversion from float-string \"65.7\" to int loses precision in ptn on line 3\n\
string(2) \"41\"\n\
string(2) \"42\"\n\
string(2) \"43\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_chr_null_and_type_diagnostics_to_native_binary() {
    let root = temp_dir("ptn-native-chr-null-and-type-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("chr-null-and-type-diagnostics.php");
    let output = root.join("chr-null-and-type-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(bin2hex(chr(null)));\n\
try { var_dump(chr(INF)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(chr(NAN)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(chr([])); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(chr(new stdClass)); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(chr(\"65x\")); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
echo \"after\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: chr(): Passing null to parameter #1 ($codepoint) of type int is deprecated in ptn on line 2\n\
string(2) \"00\"\n\
chr(): Argument #1 ($codepoint) must be of type int, float given\n\
chr(): Argument #1 ($codepoint) must be of type int, float given\n\
chr(): Argument #1 ($codepoint) must be of type int, array given\n\
chr(): Argument #1 ($codepoint) must be of type int, stdClass given\n\
chr(): Argument #1 ($codepoint) must be of type int, string given\n\
after\n"
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
        "\nDeprecated: ord(): Providing an empty string is deprecated in ptn on line 3\nint(0)\n\nDeprecated: ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead in ptn on line 4\nint(72)\n"
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
        "\nDeprecated: Invalid characters passed for attempted conversion, these have been ignored in ptn on line 3\nint(255)\n"
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
fn compile_intval_base_conversion_range_and_prefix_to_native_binary() {
    let root = temp_dir("ptn-native-intval-base-conversion-range");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("intval-base-conversion-range.php");
    let output = root.join("intval-base-conversion-range-bin");
    fs::write(
        &input,
        "<?php
var_dump(intval(\"8000000000000000\", 16));
var_dump(intval(\"ffffffffffffffff\", 16));
var_dump(intval(\"-8000000000000000\", 16));
var_dump(intval(\"0b101\", 0));
var_dump(intval(\"0B101\", 2));
var_dump(intval(\"10\", 1));
var_dump(intval(\"zz\", 36));
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(9223372036854775807)\nint(9223372036854775807)\nint(-9223372036854775808)\nint(5)\nint(5)\nint(0)\nint(1295)\n"
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
fn compile_floorceil_numeric_argument_parity_to_native_binary() {
    let root = temp_dir("ptn-native-floorceil-numeric-arguments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("floorceil-numeric-arguments.php");
    let output = root.join("floorceil-numeric-arguments-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(ceil(null));\n\
var_dump(floor(null));\n\
foreach ([true, false, \"10\", \"10.5\", \"10 \", \" 10\", \".5\", \"-.5\", \"1e309\", \"-1e309\"] as $value) {\n\
    var_dump(ceil($value));\n\
}\n\
foreach ([\"10x\", \"x10\", \"\", \" \", \"0x10\", \"INF\", [], [1]] as $value) {\n\
    try {\n\
        var_dump(floor($value));\n\
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
        "\nDeprecated: ceil(): Passing null to parameter #1 ($num) of type int|float is deprecated in ptn on line 2\nfloat(0)\n\
\nDeprecated: floor(): Passing null to parameter #1 ($num) of type int|float is deprecated in ptn on line 3\nfloat(0)\n\
float(1)\nfloat(0)\nfloat(10)\nfloat(11)\nfloat(10)\nfloat(10)\nfloat(1)\nfloat(-0)\nfloat(INF)\nfloat(-INF)\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, string given\n\
floor(): Argument #1 ($num) must be of type int|float, array given\n\
floor(): Argument #1 ($num) must be of type int|float, array given\n\
after\n"
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
fn compile_class_method_magic_constants_to_native_binary() {
    let root = temp_dir("ptn-native-class-method-magic-constants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("class-method-magic-constants.php");
    let output = root.join("class-method-magic-constants-bin");
    fs::write(
        &input,
        "<?php
class MixedCase {
    public function Run() {
        var_dump(__FUNCTION__, __METHOD__, __CLASS__, __TRAIT__, __NAMESPACE__);
    }

    public static function StaticRun() {
        var_dump(__FUNCTION__, __METHOD__, __CLASS__, __TRAIT__, __NAMESPACE__);
    }
}

$object = new MixedCase();
$object->Run();
MixedCase::StaticRun();
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(3) \"Run\"\n",
            "string(14) \"MixedCase::Run\"\n",
            "string(9) \"MixedCase\"\n",
            "string(0) \"\"\n",
            "string(0) \"\"\n",
            "string(9) \"StaticRun\"\n",
            "string(20) \"MixedCase::StaticRun\"\n",
            "string(9) \"MixedCase\"\n",
            "string(0) \"\"\n",
            "string(0) \"\"\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_string(\"Run\")"));
    assert!(c_source.contains("ptn_string(\"MixedCase::Run\")"));
    assert!(c_source.contains("ptn_string(\"StaticRun\")"));
    assert!(c_source.contains("ptn_string(\"MixedCase::StaticRun\")"));
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
fn compile_pow_function_to_native_binary() {
    let root = temp_dir("ptn-native-pow-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("pow-function.php");
    let output = root.join("pow-function-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(pow(2, 10));\n\
var_dump(pow(\"2\", \"3\"));\n\
var_dump(pow(9, 0.5));\n\
var_dump(function_exists(\"pow\"), function_exists(\"POW\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1024)\nint(8)\nfloat(3)\nbool(true)\nbool(true)\n"
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
        "<?php var_dump(function_exists(\"php_sapi_name\"), function_exists(\"PHPVERSION\"), function_exists(\"ZEND_VERSION\"), function_exists(\"get_loaded_extensions\"), PHP_SAPI, PHP_VERSION, defined(\"PHP_SAPI\"), constant(\"PHP_VERSION\"), phpversion(\"STANDARD\"), phpversion(\"missing_extension\"), zend_version()); echo implode(',', get_loaded_extensions()), \"\\n\"; var_dump(get_loaded_extensions(true));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nstring(3) \"cli\"\nstring(5) \"8.4.0\"\nbool(true)\nstring(5) \"8.4.0\"\nstring(5) \"8.4.0\"\nbool(false)\nstring(5) \"4.4.0\"\nCore,date,pcre,standard\narray(0) {\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_php_runner_environment_probes_to_native_binary() {
    let root = temp_dir("ptn-native-runner-environment-probes");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("runner-environment-probes.php");
    let output = root.join("runner-environment-probes-bin");
    fs::write(
        &input,
        "<?php\n\
echo PHP_SAPI, \"\\n\";\n\
echo PHP_VERSION, \"\\n\";\n\
echo PHP_OS, \"\\n\";\n\
echo PHP_SHLIB_SUFFIX, \"\\n\";\n\
echo php_uname('s'), \"\\n\";\n\
echo zend_version(), \"\\n\";\n\
echo ini_get('date.timezone'), \"\\n\";\n\
echo ini_get('extension_dir'), \"\\n\";\n\
var_dump(get_cfg_var('cfg_file_path'));\n\
echo php_ini_scanned_files(), \"\\n\";\n\
var_dump(extension_loaded('standard'), extension_loaded('missing'));\n\
var_dump(in_array('standard', get_loaded_extensions()), in_array('Core', get_loaded_extensions()));\n\
echo str_replace(\"\\n\", \"\", \"a\\nb\"), \"\\n\";\n\
var_dump(realpath('/definitely/not/a/ptn/path'));\n\
var_dump(function_exists('zend_version'), function_exists('ini_get'), function_exists('get_loaded_extensions'), function_exists('extension_loaded'));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let php_os = if cfg!(windows) {
        "WINNT"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };
    let shlib_suffix = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "cli\n8.4.0\n{php_os}\n{shlib_suffix}\n{php_os}\n4.4.0\nUTC\n.\nbool(false)\n\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nab\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_locale_constants_and_setlocale_to_native_binary() {
    let root = temp_dir("ptn-native-locale-constants-setlocale");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("locale-constants-setlocale.php");
    let output = root.join("locale-constants-setlocale-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(function_exists('setlocale'), defined('LC_ALL'), defined('LC_NUMERIC'));\n\
echo gettype(LC_ALL), ' ', gettype(constant('LC_CTYPE')), ' ', gettype(LC_MESSAGES), \"\\n\";\n\
var_dump(setlocale(LC_ALL, 'C'));\n\
var_dump(setlocale(LC_ALL, 0));\n\
var_dump(setlocale(LC_NUMERIC, 'POSIX'));\n\
var_dump(setlocale(LC_ALL, ['missing_ptn_locale', 'C']));\n\
var_dump(setlocale(LC_ALL, 'missing_ptn_locale'));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nbool(true)\ninteger integer integer\nstring(1) \"C\"\nstring(1) \"C\"\nstring(1) \"C\"\nstring(1) \"C\"\nbool(false)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scandir_and_preg_match_runner_probe_to_native_binary() {
    let root = temp_dir("ptn-native-scandir-preg-match-runner-probe");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("foo.so"), "").unwrap();
    fs::write(root.join("note.txt"), "").unwrap();
    fs::write(root.join("php_bar.so"), "").unwrap();
    let input = root.join("scandir-preg-match-runner-probe.php");
    let output = root.join("scandir-preg-match-runner-probe-bin");
    let source = format!(
        "<?php\n\
$names = [];\n\
foreach (scandir('{}') as $file) {{\n\
    if (preg_match('/^(?:php_)?([_a-zA-Z0-9]+)\\.(?:' . PHP_SHLIB_SUFFIX . ')$/', $file, $matches)) {{\n\
        $names[] = $matches[1];\n\
    }}\n\
}}\n\
echo implode(',', $names), \"\\n\";\n\
var_dump(preg_match('/^\\d+$/', '123'));\n\
var_dump(function_exists('scandir'), function_exists('preg_match'));\n",
        root.to_string_lossy()
    );
    fs::write(&input, source).unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "foo,bar\nint(1)\nbool(true)\nbool(true)\n"
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
fn compile_assert_internal_to_native_binary() {
    let root = temp_dir("ptn-native-assert-internal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("assert-internal.php");
    let output = root.join("assert-internal-bin");
    fs::write(
        &input,
        "<?php
var_dump(function_exists(\"assert\"));
var_dump(assert(true));
try {
    assert(false && ($a **= 2));
} catch (AssertionError $e) {
    echo 'assert(): ', $e->getMessage(), ' failed', \"\\n\";
}
if (isset($a)) {
    echo \"bad\\n\";
} else {
    echo \"short\\n\";
}
try {
    assert(false, \"custom failure\");
} catch (\\Error $e) {
    echo $e->getMessage(), \"\\n\";
}
try {
    call_user_func(\"assert\", false);
} catch (AssertionError $e) {
    var_dump($e->getMessage());
}
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(true)\nassert(): assert(false && ($a **= 2)) failed\nshort\ncustom failure\nstring(0) \"\"\n"
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
fn compile_define_legacy_case_insensitive_flag_is_ignored_with_warning() {
    let root = temp_dir("ptn-native-define-legacy-case-flag");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("define-legacy-case-flag.php");
    let output = root.join("define-legacy-case-flag-bin");
    fs::write(
        &input,
        "<?php\n\
function marker($label, $value) { echo $label, \"\\n\"; return $value; }\n\
var_dump(define(marker(\"name\", \"CASE_ARG\"), marker(\"value\", 9), marker(\"flag\", true)));\n\
var_dump(defined(\"CASE_ARG\"), defined(\"case_arg\"), constant(\"CASE_ARG\"));\n\
define(\"DUP_ARG\", 1);\n\
var_dump(define(marker(\"dup-name\", \"DUP_ARG\"), marker(\"dup-value\", 2), marker(\"dup-flag\", true)));\n\
var_dump(constant(\"DUP_ARG\"));\n\
var_dump(define(\"FALSE_FLAG\", 5, false), defined(\"false_flag\"), constant(\"FALSE_FLAG\"));\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "name\n\
value\n\
flag\n\
Warning: define(): Argument #3 ($case_insensitive) is ignored since declaration of case-insensitive constants is no longer supported in ptn on line 3\n\
bool(true)\n\
bool(true)\n\
bool(false)\n\
int(9)\n\
dup-name\n\
dup-value\n\
dup-flag\n\
Warning: define(): Argument #3 ($case_insensitive) is ignored since declaration of case-insensitive constants is no longer supported in ptn on line 6\n\
Warning: Constant DUP_ARG already defined, this will be an error in PHP 9 in ptn on line 6\n\
bool(false)\n\
int(1)\n\
bool(true)\n\
bool(false)\n\
int(5)\n"
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
fn compile_comparison_operands_drop_cow_payloads_to_native_binary() {
    let root = temp_dir("ptn-native-comparison-cow-cleanup");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("comparison-cow-cleanup.php");
    let output = root.join("comparison-cow-cleanup-bin");
    fs::write(
        &input,
        "<?php\n\
_ptn_cow_debug_reset();\n\
var_dump([1, [\"x\" => \"y\"]] == [1, [\"x\" => \"y\"]]);\n\
var_dump([\"k\" => \"v\"] !== [\"k\" => \"v\"]);\n\
var_dump((\"a\" . \"b\") == (\"a\" . \"b\"));\n\
_ptn_cow_debug_assert_counter(\"array.live\", 0);\n\
_ptn_cow_debug_assert_counter(\"string.live\", 0);\n\
_ptn_cow_debug_assert_balanced();\n\
echo _ptn_cow_debug_counter(\"array.live\"), \":\", _ptn_cow_debug_counter(\"string.live\"), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nbool(true)\n0:0\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert!(main_body.contains(" = ptn_bool(ptn_compare_equal("));
    assert!(main_body.contains(" = ptn_bool(ptn_compare_not_identical("));
    assert!(main_body.contains("ptn_value_drop(&ptn_tmp_"));
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
fn compile_ternary_expressions_to_native_binary() {
    let root = temp_dir("ptn-native-ternary-expressions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("ternary-expressions.php");
    let output = root.join("ternary-expressions-bin");
    fs::write(
        &input,
        "<?php
function fail() {
    echo \"bad\";
    return \"bad\";
}

echo true ? \"yes\" : fail(), \"\\n\";
echo false ? fail() : \"no\", \"\\n\";
$value = \"keep\";
echo $value ?: fail(), \"\\n\";
$value = \"\";
echo $value ?: \"fallback\", \"\\n\";
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "yes\nno\nkeep\nfallback\n"
    );
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
        format!(
            "bool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\n{}bool(false)\n",
            undefined_variable_warnings(&input, &[("left", 10), ("right", 10)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_keyword_boolean_assignment_tails_to_native_binary() {
    let root = temp_dir("ptn-native-keyword-boolean-assignment-tails");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("keyword-boolean-assignment-tails.php");
    let output = root.join("keyword-boolean-assignment-tails-bin");
    fs::write(
        &input,
        "<?php\n\
$a = true and false;\n\
var_dump($a);\n\
$b = false or true;\n\
var_dump($b);\n\
$c = true xor false;\n\
var_dump($c);\n\
$hit = 0;\n\
$d = false and $hit = 1;\n\
var_dump($d, $hit);\n\
$e = true or $hit = 2;\n\
var_dump($e, $hit);\n\
$items = [0];\n\
$items[0] = true and false;\n\
var_dump($items[0]);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nbool(true)\nbool(false)\nint(0)\nbool(true)\nint(0)\nbool(true)\n"
    );
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
fn compile_count_modes_and_sizeof_alias_to_native_binary() {
    let root = temp_dir("ptn-native-count-modes-sizeof-alias");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("count-modes-sizeof-alias.php");
    let output = root.join("count-modes-sizeof-alias-bin");
    fs::write(
        &input,
        "<?php\n\
$nested = [1, [2, 3], [\"x\" => [4]]];\n\
var_dump(COUNT_NORMAL, COUNT_RECURSIVE);\n\
var_dump(count($nested), count($nested, COUNT_NORMAL), count($nested, 0));\n\
var_dump(count($nested, COUNT_RECURSIVE), count($nested, 1));\n\
var_dump(sizeof($nested), sizeof($nested, COUNT_RECURSIVE));\n\
try { count($nested, 2); } catch (\\ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { sizeof($nested, -1); } catch (\\ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { sizeof(false); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists(\"sizeof\"), function_exists(\"SIZEOF\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(0)\nint(1)\nint(3)\nint(3)\nint(3)\nint(7)\nint(7)\nint(3)\nint(7)\ncount(): Argument #2 ($mode) must be either COUNT_NORMAL or COUNT_RECURSIVE\nsizeof(): Argument #2 ($mode) must be either COUNT_NORMAL or COUNT_RECURSIVE\nsizeof(): Argument #1 ($value) must be of type Countable|array, false given\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_count"));
    assert!(c_source.contains("ptn_internal_sizeof"));
}

#[test]
fn compile_object_method_callable_without_declared_methods_to_native_binary() {
    let root = temp_dir("ptn-native-object-method-callable-no-declared-methods");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("object-method-callable-no-declared-methods.php");
    let output = root.join("object-method-callable-no-declared-methods-bin");
    fs::write(
        &input,
        "<?php
try {
    count(null);
} catch (\\TypeError $e) {
    echo call_user_func([$e, \"getMessage\"]), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "count(): Argument #1 ($value) must be of type Countable|array, null given\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_call_user_func"));
    assert!(c_source.contains("ptn_call_declared_method(runtime, receiver"));
    assert!(c_source.contains("ptn_call_method(runtime, resolved, method_name"));
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
fn compile_foreach_array_dim_value_target_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-array-dim-value");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-array-dim-value.php");
    let output = root.join("foreach-array-dim-value-bin");
    fs::write(
        &input,
        "<?php\n\
$a = [\"a\", \"b\", \"c\"];\n\
$v = [];\n\
foreach ($a as $v[0]) {\n\
    var_dump($v);\n\
}\n\
var_dump($a);\n\
var_dump($v);\n\
\n\
echo \"\\n\";\n\
$a = [\"a\", \"b\", \"c\"];\n\
$v = [];\n\
foreach ($a as $k => $v[0]) {\n\
    var_dump($k, $v);\n\
}\n\
var_dump($a);\n\
var_dump($k, $v);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(1) {\n  [0]=>\n  string(1) \"a\"\n}\n\
array(1) {\n  [0]=>\n  string(1) \"b\"\n}\n\
array(1) {\n  [0]=>\n  string(1) \"c\"\n}\n\
array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"c\"\n}\n\
array(1) {\n  [0]=>\n  string(1) \"c\"\n}\n\
\n\
int(0)\narray(1) {\n  [0]=>\n  string(1) \"a\"\n}\n\
int(1)\narray(1) {\n  [0]=>\n  string(1) \"b\"\n}\n\
int(2)\narray(1) {\n  [0]=>\n  string(1) \"c\"\n}\n\
array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"c\"\n}\n\
int(2)\narray(1) {\n  [0]=>\n  string(1) \"c\"\n}\n"
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
fn compile_foreach_empty_statement_body_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-empty-body");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-empty-body.php");
    let output = root.join("foreach-empty-body-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"a\", \"b\", \"c\"];\n\
foreach ($items as $value);\n\
var_dump($value);\n\
foreach ($items as $key => $value);\n\
var_dump($key, $value);\n\
echo \"done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"c\"\nint(2)\nstring(1) \"c\"\ndone\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_non_array_diagnostics_include_source_path_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-non-array-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-non-array-diagnostics.php");
    let output = root.join("foreach-non-array-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
foreach (false as $value) { echo \"bad\"; }\n\
foreach (true as $value) { echo \"bad\"; }\n\
$scalar = \"x\";\n\
foreach ($scalar as &$value) { echo \"bad\"; }\n\
echo \"done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "\nWarning: foreach() argument must be of type array|object, false given in {} on line 2\n\
\nWarning: foreach() argument must be of type array|object, true given in {} on line 3\n\
\nWarning: foreach() argument must be of type array|object, string given in {} on line 5\n\
done\n",
            input.display(),
            input.display(),
            input.display()
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_object_properties_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-object-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-object-properties.php");
    let output = root.join("foreach-object-properties-bin");
    fs::write(
        &input,
        "<?php\n\
$object = new stdClass;\n\
$object->a = 1;\n\
$object->b = \"two\";\n\
foreach ($object as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo \"done\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "a=1\nb=two\ndone\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_object_properties_are_live_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-object-live-properties");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-object-live-properties.php");
    let output = root.join("foreach-object-live-properties-bin");
    fs::write(
        &input,
        "<?php\n\
$object = new stdClass;\n\
$object->a = 1;\n\
foreach ($object as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
    if ($key === \"a\") {\n\
        $object->b = 2;\n\
    }\n\
}\n\
foreach ($object as $key => &$value) {\n\
    $value += 10;\n\
}\n\
echo $object->a, \":\", $object->b, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "a=1\nb=2\n11:12\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_by_value_snapshots_iteration_set_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-cow-snapshot");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-cow-snapshot.php");
    let output = root.join("foreach-cow-snapshot-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
foreach ($items as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
    if ($key === \"a\") {\n\
        unset($items[\"a\"]);\n\
        unset($items[\"b\"]);\n\
        $items[] = \"D\";\n\
    }\n\
    if ($key === \"b\") {\n\
        unset($items[\"c\"]);\n\
    }\n\
}\n\
var_dump($items);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "a=A\n",
            "b=B\n",
            "c=C\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(1) \"D\"\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_array_retain(iterator.array);"));
    assert!(c_source.contains("ptn_array_iterator_destroy(&"));
    assert!(c_source.contains("ptn_foreach_cleanup"));
}

#[test]
fn compile_foreach_by_value_detaches_alias_mutations_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-cow-aliases");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-cow-aliases.php");
    let output = root.join("foreach-cow-aliases-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"x\" => 1, \"y\" => 2];\n\
$alias = $source;\n\
foreach ($source as $key => $value) {\n\
    echo $key, \":\", $value, \"\\n\";\n\
    if ($key === \"x\") {\n\
        $source[\"z\"] = 3;\n\
        $alias[\"y\"] = 20;\n\
    }\n\
}\n\
var_dump($source);\n\
var_dump($alias);\n\
$post = $source;\n\
$post[] = 4;\n\
var_dump($source);\n\
var_dump($post);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "x:1\n",
            "y:2\n",
            "array(3) {\n",
            "  [\"x\"]=>\n",
            "  int(1)\n",
            "  [\"y\"]=>\n",
            "  int(2)\n",
            "  [\"z\"]=>\n",
            "  int(3)\n",
            "}\n",
            "array(2) {\n",
            "  [\"x\"]=>\n",
            "  int(1)\n",
            "  [\"y\"]=>\n",
            "  int(20)\n",
            "}\n",
            "array(3) {\n",
            "  [\"x\"]=>\n",
            "  int(1)\n",
            "  [\"y\"]=>\n",
            "  int(2)\n",
            "  [\"z\"]=>\n",
            "  int(3)\n",
            "}\n",
            "array(4) {\n",
            "  [\"x\"]=>\n",
            "  int(1)\n",
            "  [\"y\"]=>\n",
            "  int(2)\n",
            "  [\"z\"]=>\n",
            "  int(3)\n",
            "  [0]=>\n",
            "  int(4)\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_temporary_reference_array_literal_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-temp-reference-array");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-temp-reference-array.php");
    let output = root.join("foreach-temp-reference-array-bin");
    fs::write(
        &input,
        "<?php\n\
$a = 'a';\n\
$b = 'b';\n\
foreach ([&$a, &$b] as &$value) {\n\
    $value .= '-foo';\n\
}\n\
var_dump($a, $b);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"a-foo\"\nstring(5) \"b-foo\"\n"
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
        "\nWarning: Undefined array key \"7.5\" in ptn on line 3\nNULL\n\nWarning: Undefined array key 0 in ptn on line 4\nNULL\n\nWarning: Trying to access array offset on int in ptn on line 6\nNULL\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_null_array_offset_diagnostics_respect_suppression_to_native_binary() {
    let root = temp_dir("ptn-native-null-array-offset-diagnostic-suppression");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("null-array-offset-diagnostic-suppression.php");
    let output = root.join("null-array-offset-diagnostic-suppression-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [];\n\
@$items[null];\n\
var_dump($items);\n\
var_dump($items[null]);\n\
@$items[null];\n\
$items[null] = \"stored\";\n\
var_dump($items[\"\"]);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 5\n\nWarning: Undefined array key \"\" in ptn on line 5\nNULL\n\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 7\nstring(6) \"stored\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_offset_assignment_aliasing_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-offset-assignment-aliasing");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-offset-assignment-aliasing.php");
    let output = root.join("scalar-offset-assignment-aliasing-bin");
    fs::write(
        &input,
        "<?php\n\
$float = 0.213123123;\n\
$float_alias =& $float;\n\
$float = $float[1];\n\
var_dump($float);\n\
var_dump($float_alias);\n\
$int = 7;\n\
$int_alias =& $int;\n\
$int = $int[0];\n\
var_dump($int);\n\
var_dump($int_alias);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nWarning: Trying to access array offset on float in ptn on line 4\nNULL\nNULL\n\nWarning: Trying to access array offset on int in ptn on line 9\nNULL\nNULL\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_offset_lvalue_write_boundaries_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-offset-lvalue-boundaries");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-offset-lvalue-boundaries.php");
    let output = root.join("scalar-offset-lvalue-boundaries-bin");
    fs::write(
        &input,
        "<?php\n\
$int = 7;\n\
try { $int[0] = \"x\"; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump($int);\n\
$num = 3;\n\
try { $num[0] += 2; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump($num);\n\
$refTarget = \"x\";\n\
$truth = true;\n\
try { $truth[0] =& $refTarget; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump($truth);\n\
$false = false;\n\
$false[0] = \"ok\";\n\
var_dump($false);\n\
$nested = [false, null, 1];\n\
$nested[0][1] = \"converted\";\n\
$nested[1][2] = \"null-converted\";\n\
try { $nested[2][3] = \"bad\"; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump($nested);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot use a scalar value as an array\nint(7)\nCannot use a scalar value as an array\nint(3)\nCannot use a scalar value as an array\nbool(true)\n\nDeprecated: Automatic conversion of false to array is deprecated in ptn on line 13\narray(1) {\n  [0]=>\n  string(2) \"ok\"\n}\n\nDeprecated: Automatic conversion of false to array is deprecated in ptn on line 16\nCannot use a scalar value as an array\narray(3) {\n  [0]=>\n  array(1) {\n    [1]=>\n    string(9) \"converted\"\n  }\n  [1]=>\n  array(1) {\n    [2]=>\n    string(14) \"null-converted\"\n  }\n  [2]=>\n  int(1)\n}\n"
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
fn compile_nested_array_cow_mutation_to_native_binary() {
    let root = temp_dir("ptn-native-nested-array-cow-mutation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-array-cow-mutation.php");
    let output = root.join("nested-array-cow-mutation-bin");
    fs::write(
        &input,
        "<?php\n\
$base = [[1, 2], [\"s\" => \"abc\"], [\"drop\" => [10]]];\n\
$alias = $base;\n\
$alias[0][1] = 99;\n\
$alias[0][] = 100;\n\
$alias[1][\"s\"] .= \"-mut\";\n\
unset($alias[2][\"drop\"]);\n\
var_dump($base[0], $alias[0], $base[1][\"s\"], $alias[1][\"s\"], isset($base[2][\"drop\"]), isset($alias[2][\"drop\"]));\n\
$cycle = [[\"x\" => [\"y\" => \"z\"]]];\n\
$copy = $cycle;\n\
$copy[0][\"x\"][\"y\"] = \"changed\";\n\
unset($copy[0][\"x\"]);\n\
$copy[0][\"x\"][] = \"new\";\n\
var_dump($cycle, $copy);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\narray(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(99)\n  [2]=>\n  int(100)\n}\nstring(3) \"abc\"\nstring(7) \"abc-mut\"\nbool(true)\nbool(false)\narray(1) {\n  [0]=>\n  array(1) {\n    [\"x\"]=>\n    array(1) {\n      [\"y\"]=>\n      string(1) \"z\"\n    }\n  }\n}\narray(1) {\n  [0]=>\n  array(1) {\n    [\"x\"]=>\n    array(1) {\n      [0]=>\n      string(3) \"new\"\n    }\n  }\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_path_self_assignment_snapshots_rhs_to_native_binary() {
    let root = temp_dir("ptn-native-array-path-self-assignment-snapshot");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-path-self-assignment-snapshot.php");
    let output = root.join("array-path-self-assignment-snapshot-bin");
    fs::write(
        &input,
        "<?php\n\
$a = [];\n\
$a[0] = $a;\n\
var_dump($a);\n\
$b = [[]];\n\
$b[0][0] = $b;\n\
var_dump($b);\n\
$source = [[\"leaf\" => [\"v\" => 1]]];\n\
$copy = $source;\n\
$read = $copy[0][\"leaf\"];\n\
$read[\"v\"] = 2;\n\
var_dump($source[0][\"leaf\"][\"v\"], $copy[0][\"leaf\"][\"v\"], $read[\"v\"]);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(1) {\n",
            "  [0]=>\n",
            "  array(0) {\n",
            "  }\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  array(1) {\n",
            "    [0]=>\n",
            "    array(1) {\n",
            "      [0]=>\n",
            "      array(0) {\n",
            "      }\n",
            "    }\n",
            "  }\n",
            "}\n",
            "int(1)\n",
            "int(1)\n",
            "int(2)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_value_snapshot_for_array_path_write"));
    assert!(c_source.contains("ptn_runtime_array_path_set(&runtime"));
}

#[test]
fn compile_reference_aliases_and_cow_split_to_native_binary() {
    let root = temp_dir("ptn-native-reference-aliases-cow-split");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("reference-aliases-cow-split.php");
    let output = root.join("reference-aliases-cow-split-bin");
    fs::write(
        &input,
        "<?php\n\
$a = [1, 2];\n\
$b = $a;\n\
$ref =& $a;\n\
$ref[0] = 9;\n\
var_dump($a[0], $b[0]);\n\
$x = 1;\n\
$y =& $x;\n\
$z = $x;\n\
$y = 2;\n\
var_dump($x, $y, $z);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(9)\nint(1)\nint(2)\nint(2)\nint(1)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_element_references_to_native_binary() {
    let root = temp_dir("ptn-native-array-element-references");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-element-references.php");
    let output = root.join("array-element-references-bin");
    fs::write(
        &input,
        "<?php\n\
$arr = [1];\n\
$copy_before = $arr;\n\
$elem =& $arr[0];\n\
$copy_after = $arr;\n\
$elem = 2;\n\
var_dump($arr[0], $copy_before[0], $copy_after[0]);\n\
$copy_after[0] = 3;\n\
var_dump($arr[0], $elem);\n\
$other = 4;\n\
$arr[0] =& $other;\n\
$other = 5;\n\
$elem = 6;\n\
var_dump($arr[0], $other, $elem);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(2)\nint(1)\nint(2)\nint(3)\nint(3)\nint(5)\nint(5)\nint(6)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_nested_recursive_reference_lvalues_to_native_binary() {
    let root = temp_dir("ptn-native-nested-recursive-reference-lvalues");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-recursive-reference-lvalues.php");
    let output = root.join("nested-recursive-reference-lvalues-bin");
    fs::write(
        &input,
        "<?php\n\
$a = array(array(1));\n\
$a[0][] =& $a[0];\n\
$a[0][] =& $a[0];\n\
$a[0][0] = 2;\n\
var_dump($a);\n\
$a[0] = null;\n\
$a = null;",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(1) {\n  [0]=>\n  &array(3) {\n    [0]=>\n    int(2)\n    [1]=>\n    *RECURSION*\n    [2]=>\n    *RECURSION*\n  }\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_grouped_reference_lvalues_to_native_binary() {
    let root = temp_dir("ptn-native-grouped-reference-lvalues");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("grouped-reference-lvalues.php");
    let output = root.join("grouped-reference-lvalues-bin");
    fs::write(
        &input,
        "<?php\n\
$value = 1;\n\
$alias =& ($value);\n\
$alias = 2;\n\
echo $value, ':', $alias, \"\\n\";\n\
$items = [10, 20];\n\
$first =& ($items)[0];\n\
$second =& ($items[1]);\n\
$first = 30;\n\
$second = 40;\n\
echo $items[0], ':', $items[1], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "2:2\n30:40\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_sum_dereferences_array_entries_to_native_binary() {
    let root = temp_dir("ptn-native-array-sum-reference-entries");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-sum-reference-entries.php");
    let output = root.join("array-sum-reference-entries-bin");
    fs::write(
        &input,
        "<?php\n\
$n = \"10\";\n\
$n .= \"0\";\n\
$nums = [&$n, 100];\n\
var_dump(array_sum($nums));\n\
var_dump($n);\n\
$f = \"1.5\";\n\
$mix = [&$f, 2];\n\
var_dump(array_sum($mix));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(200)\nstring(3) \"100\"\nfloat(3.5)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_product_dereferences_array_entries_to_native_binary() {
    let root = temp_dir("ptn-native-array-product-reference-entries");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-product-reference-entries.php");
    let output = root.join("array-product-reference-entries-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_product([]));\n\
$n = \"10\";\n\
$n .= \"0\";\n\
$nums = [&$n, 2];\n\
var_dump(array_product($nums));\n\
var_dump($n);\n\
$f = \"1.5\";\n\
$mix = [&$f, 2, 4];\n\
var_dump(array_product($mix));\n\
var_dump(function_exists('array_product'), function_exists('ARRAY_PRODUCT'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(200)\nstring(3) \"100\"\nfloat(12)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_sum_product_warning_and_overflow_parity_to_native_binary() {
    let root = temp_dir("ptn-native-array-sum-product-warning-overflow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-sum-product-warning-overflow.php");
    let output = root.join("array-sum-product-warning-overflow-bin");
    fs::write(
        &input,
        "<?php\n\
class Box { public $value; public function __construct($value) { $this->value = $value; } }\n\
var_dump(defined(\"STDERR\"), STDERR);\n\
var_dump(array_sum([\"apple\", [1], new Box(1), STDERR]));\n\
var_dump(array_product([[1], new Box(1), STDERR]));\n\
var_dump(is_float(array_sum([PHP_INT_MAX, 1])));\n\
var_dump(is_float(array_product([PHP_INT_MAX, 2])));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\n\
resource(3) of type (stream)\n\
\n\
Warning: array_sum(): Addition is not supported on type string in ptn on line 4\n\
\n\
Warning: array_sum(): Addition is not supported on type array in ptn on line 4\n\
\n\
Warning: array_sum(): Addition is not supported on type Box in ptn on line 4\n\
\n\
Warning: array_sum(): Addition is not supported on type resource in ptn on line 4\n\
int(3)\n\
\n\
Warning: array_product(): Multiplication is not supported on type array in ptn on line 5\n\
\n\
Warning: array_product(): Multiplication is not supported on type Box in ptn on line 5\n\
\n\
Warning: array_product(): Multiplication is not supported on type resource in ptn on line 5\n\
int(3)\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_array_aggregate_number"));
    assert!(c_source.contains("ptn_int64_multiply_overflows"));
    assert!(c_source.contains("ptn_standard_stream_resource_value"));
}

#[test]
fn compile_strtr_dereferences_replacement_array_entries_to_native_binary() {
    let root = temp_dir("ptn-native-strtr-reference-map");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("strtr-reference-map.php");
    let output = root.join("strtr-reference-map-bin");
    fs::write(
        &input,
        "<?php\n\
$foo = \"foo\";\n\
$arr = [\"bar\" => &$foo, \"foobar\" => \"whole\"];\n\
var_dump(strtr(\"foobarbar\", $arr));\n\
$foo = \"baz\";\n\
var_dump(strtr(\"bar\", $arr));\n\
var_dump(strtr(\"abc\", \"ab\", \"xy\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(8) \"wholefoo\"\nstring(3) \"baz\"\nstring(3) \"xyc\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_in_array_dereferences_needle_and_haystack_entries_to_native_binary() {
    let root = temp_dir("ptn-native-in-array-reference-aware");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("in-array-reference-aware.php");
    let output = root.join("in-array-reference-aware-bin");
    fs::write(
        &input,
        "<?php\n\
$value = \"10\";\n\
$haystack = [&$value, 20, \"030\"];\n\
var_dump(in_array(10, $haystack));\n\
var_dump(in_array(10, $haystack, true));\n\
var_dump(in_array(\"10\", $haystack, true));\n\
$value = 30;\n\
var_dump(in_array(\"30\", $haystack));\n\
var_dump(in_array(\"30\", $haystack, true));\n\
$needle =& $value;\n\
var_dump(in_array($needle, $haystack, true));\n\
var_dump(function_exists(\"in_array\"), function_exists(\"IN_ARRAY\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_search_returns_first_matching_key_to_native_binary() {
    let root = temp_dir("ptn-native-array-search");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-search.php");
    let output = root.join("array-search-bin");
    fs::write(
        &input,
        "<?php\n\
$value = \"10\";\n\
$haystack = [\"first\" => &$value, 20, \"030\", \"two\" => 2];\n\
var_dump(array_search(10, $haystack));\n\
var_dump(array_search(10, $haystack, true));\n\
var_dump(array_search(\"10\", $haystack, true));\n\
$value = 30;\n\
var_dump(array_search(\"30\", $haystack));\n\
var_dump(array_search(\"30\", $haystack, true));\n\
$needle =& $value;\n\
var_dump(array_search($needle, $haystack, true));\n\
var_dump(array_search(2, $haystack));\n\
var_dump(array_search(\"2\", $haystack, true));\n\
var_dump(array_search(\"missing\", $haystack));\n\
var_dump(function_exists(\"array_search\"), function_exists(\"ARRAY_SEARCH\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"first\"\nbool(false)\nstring(5) \"first\"\nstring(5) \"first\"\nbool(false)\nstring(5) \"first\"\nstring(3) \"two\"\nbool(false)\nbool(false)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_search"));
    assert!(c_source.contains("ptn_compare_equal"));
    assert!(c_source.contains("ptn_compare_identical"));
}

#[test]
fn compile_array_literal_reference_elements_to_native_binary() {
    let root = temp_dir("ptn-native-array-literal-reference-elements");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-literal-reference-elements.php");
    let output = root.join("array-literal-reference-elements-bin");
    fs::write(
        &input,
        "<?php\n\
$value = \"one\";\n\
$refs = [&$value];\n\
$refs[0] = \"two\";\n\
echo $value, \":\", $refs[0], \"\\n\";\n\
$items = [\"a\"];\n\
$keyed = [\"k\" => &$items[0]];\n\
$keyed[\"k\"] = \"b\";\n\
echo $items[0], \":\", $keyed[\"k\"], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "two:two\nb:b\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_append_expression_with_reference_list_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-append-reference-list-assignment-expression");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("append-reference-list-assignment-expression.php");
    let output = root.join("append-reference-list-assignment-expression-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump($ary[] = [&$x] = $x);\n\
var_dump($x);\n\
var_dump($ary);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(1) {\n  [0]=>\n  &NULL\n}\nNULL\narray(1) {\n  [0]=>\n  array(1) {\n    [0]=>\n    &NULL\n  }\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_list_assignment_expression_result_appends_to_native_binary() {
    let root = temp_dir("ptn-native-list-assignment-expression-result-append");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("list-assignment-expression-result-append.php");
    let output = root.join("list-assignment-expression-result-append-bin");
    fs::write(
        &input,
        "<?php\n\
$rhs = [1, 2];\n\
$ary[] = [$a, $b] = $rhs;\n\
echo $a, \":\", $b, \":\", $ary[0][1], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1:2:2\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_long_list_assignment_statement_to_native_binary() {
    let root = temp_dir("ptn-native-long-list-assignment-statement");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("long-list-assignment-statement.php");
    let output = root.join("long-list-assignment-statement-bin");
    fs::write(
        &input,
        "<?php\n\
$rhs = [\"left\", \"right\"];\n\
list($a, $b) = $rhs;\n\
echo $a, \":\", $b, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "left:right\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_reference_list_assignment_binds_rhs_array_slot_to_native_binary() {
    let root = temp_dir("ptn-native-reference-list-assignment-slot");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("reference-list-assignment-slot.php");
    let output = root.join("reference-list-assignment-slot-bin");
    fs::write(
        &input,
        "<?php\n\
$x = \"one\";\n\
$rhs = [&$x, \"extra\"];\n\
$result = [&$alias] = $rhs;\n\
$x = \"two\";\n\
echo $alias, \":\", $result[0], \":\", $result[1], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "two:two:extra\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_foreach_list_destructuring_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-list-destructuring");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-list-destructuring.php");
    let output = root.join("foreach-list-destructuring-bin");
    fs::write(
        &input,
        "<?php\n\
$pairs = [[\"left\", [\"inner\"]], [\"right\", [\"next\"]]];\n\
foreach ($pairs as [$a, [$b]]) {\n\
    echo $a, \":\", $b, \"\\n\";\n\
}\n\
$keyed = [[\"name\" => \"Ada\", \"meta\" => [\"id\" => 10]], [\"name\" => \"Lin\", \"meta\" => [\"id\" => 20]]];\n\
foreach ($keyed as [\"name\" => $name, \"meta\" => [\"id\" => $id]]) {\n\
    echo $name, \"#\", $id, \"\\n\";\n\
}\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "left:inner\nright:next\nAda#10\nLin#20\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_array_iterator_current_value"));
    assert!(c_source.contains("ptn_array_read(&runtime"));
}

#[test]
fn compile_foreach_list_reference_elements_to_native_binary() {
    let root = temp_dir("ptn-native-foreach-list-reference-elements");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-list-reference-elements.php");
    let output = root.join("foreach-list-reference-elements-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [[1], [2]];\n\
foreach ($items as [&$slot]) {\n\
    $slot += 10;\n\
}\n\
unset($slot);\n\
var_dump($items);\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [0]=>\n  array(1) {\n    [0]=>\n    int(11)\n  }\n  [1]=>\n  array(1) {\n    [0]=>\n    int(12)\n  }\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_reference_for_array_value_dim"));
    assert!(c_source.contains("ptn_runtime_bind_variable_reference(&runtime, \"slot\""));
}

#[test]
fn compile_reference_array_literals_and_internals_to_native_binary() {
    let root = temp_dir("ptn-native-reference-array-literals-internals");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("reference-array-literals-internals.php");
    let output = root.join("reference-array-literals-internals-bin");
    fs::write(
        &input,
        "<?php\n\
$foo = 42;\n\
$array1 = [&$foo];\n\
$array2 = [$foo];\n\
var_dump($array1 === $array2);\n\
$n = \"10\";\n\
$n .= \"0\";\n\
$nums = [&$n, 100];\n\
var_dump(array_sum($nums));\n\
var_dump($n);\n\
$word = \"foo\";\n\
$map = [\"bar\" => &$word];\n\
var_dump(strtr(\"foobar\", $map));\n\
$r = 1;\n\
$a = [&$r];\n\
debug_zval_dump($a);\n\
$a[] =& $r;\n\
debug_zval_dump($a);\n\
unset($a[1]);\n\
debug_zval_dump($a);\n\
unset($r);\n\
debug_zval_dump($a);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nint(200)\nstring(3) \"100\"\nstring(6) \"foofoo\"\narray(1) packed refcount(2){\n  [0]=>\n  reference refcount(2) {\n    int(1)\n  }\n}\narray(2) packed refcount(2){\n  [0]=>\n  reference refcount(3) {\n    int(1)\n  }\n  [1]=>\n  reference refcount(3) {\n    int(1)\n  }\n}\narray(1) packed refcount(2){\n  [0]=>\n  reference refcount(2) {\n    int(1)\n  }\n}\narray(1) packed refcount(2){\n  [0]=>\n  reference refcount(1) {\n    int(1)\n  }\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_offset_reference_array_literal_raises_error_to_native_binary() {
    let root = temp_dir("ptn-native-string-offset-reference-array-literal");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-offset-reference-array-literal.php");
    let output = root.join("string-offset-reference-array-literal-bin");
    fs::write(
        &input,
        "<?php\n\
$text = \"abc\";\n\
try { $refs = [&$text[1]]; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
echo $text, \"\\n\";\n\
function string_offset_ref_in_function() {\n\
    $inner = \"\";\n\
    return array(&$inner[0]);\n\
}\n\
try { string_offset_ref_in_function(); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot create references to/from string offsets\nabc\nCannot create references to/from string offsets\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_reference_parameters_to_native_binary() {
    let root = temp_dir("ptn-native-reference-parameters");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("reference-parameters.php");
    let output = root.join("reference-parameters-bin");
    fs::write(
        &input,
        "<?php\n\
function bump(&$value) { $value += 1; return $value; }\n\
function see($value) { $value += 10; return $value; }\n\
function snapshot(&$value) { $args = func_get_args(); $value = 9; var_dump($args[0], func_get_arg(0)); }\n\
$n = 1;\n\
var_dump(bump($n), $n, see($n), $n);\n\
$items = [1];\n\
var_dump(bump($items[0]), $items[0]);\n\
$m = 7;\n\
snapshot($m);\n\
var_dump($m);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(2)\nint(2)\nint(12)\nint(2)\nint(2)\nint(2)\nint(7)\nint(9)\nint(9)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_by_reference_default_parameter_to_native_binary() {
    let root = temp_dir("ptn-native-by-reference-default-parameter");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-reference-default-parameter.php");
    let output = root.join("by-reference-default-parameter-bin");
    fs::write(
        &input,
        "<?php\n\
function maybe(&$value = null) { var_dump($value); $value = 9; }\n\
maybe();\n\
$x = 1;\n\
maybe($x);\n\
var_dump($x);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "NULL\nint(1)\nint(9)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_by_value_parameters_near_references_to_native_binary() {
    let root = temp_dir("ptn-native-by-value-parameters-near-references");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-value-parameters-near-references.php");
    let output = root.join("by-value-parameters-near-references-bin");
    fs::write(
        &input,
        "<?php\n\
function change_value($value) { $value = 99; return $value; }\n\
function change_array_value($value) { $value[0] = 7; return $value[0]; }\n\
function change_array_ref_element($value) { $value[0] = 8; return $value[0]; }\n\
$q = 1;\n\
$qr =& $q;\n\
var_dump(change_value($q), $q);\n\
$plain = [1];\n\
$plain_ref =& $plain;\n\
var_dump(change_array_value($plain), $plain[0]);\n\
$with_ref = [1];\n\
$leaf =& $with_ref[0];\n\
var_dump(change_array_ref_element($with_ref), $with_ref[0], $leaf);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(99)\nint(1)\nint(7)\nint(1)\nint(8)\nint(8)\nint(8)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_by_reference_argument_diagnostic_to_native_binary() {
    let root = temp_dir("ptn-native-by-reference-argument-diagnostic");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-reference-argument-diagnostic.php");
    let output = root.join("by-reference-argument-diagnostic-bin");
    fs::write(
        &input,
        "<?php function takes_ref(&$value) { $value = 1; } takes_ref(1 + 2);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: takes_ref(): Argument #1 ($value) cannot be passed by reference\n"
    );
}

#[test]
fn compile_by_reference_assignment_call_result_value_fallback_to_native_binary() {
    let root = temp_dir("ptn-native-by-reference-assignment-call-result-value-fallback");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-reference-assignment-call-result-value-fallback.php");
    let output = root.join("by-reference-assignment-call-result-value-fallback-bin");
    fs::write(
        &input,
        "<?php\n\
function value() { return 1; }\n\
$alias =& value();\n\
echo $alias, \"\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Notice: Only variables should be assigned by reference in ptn on line 3\n1\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_by_reference_return_call_result_chains_to_native_binary() {
    let root = temp_dir("ptn-native-by-reference-return-call-result-chains");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-reference-return-call-result-chains.php");
    let output = root.join("by-reference-return-call-result-chains-bin");
    fs::write(
        &input,
        "<?php\n\
function &id(&$value) {\n\
    return $value;\n\
}\n\
function &chain(&$value) {\n\
    return id($value);\n\
}\n\
function make_value() {\n\
    return 9;\n\
}\n\
function &value_chain() {\n\
    return make_value();\n\
}\n\
function &typed_value_chain(): string {\n\
    return make_value();\n\
}\n\
$value = 1;\n\
$alias =& chain($value);\n\
$alias = 2;\n\
echo $value, \"|\", $alias, \"\\n\";\n\
$copy = chain($value);\n\
$copy = 3;\n\
echo $value, \"|\", $copy, \"\\n\";\n\
$fallback =& value_chain();\n\
echo $fallback, \"\\n\";\n\
$typed =& typed_value_chain();\n\
echo gettype($typed), \":\", $typed, \"\\n\";\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2|2\n\
2|3\n\
Notice: Only variable references should be returned by reference in ptn on line 12\n\
9\n\
Notice: Only variable references should be returned by reference in ptn on line 15\n\
string:9\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_reference_source_or_value(&runtime, "));
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
        format!(
            "{}\nWarning: Undefined array key 0 in ptn on line 2\nint(2)\n{}string(1) \"x\"\n{}\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line 6\n\nWarning: Undefined array key \"\" in ptn on line 6\narray(1) {{\n  [\"\"]=>\n  int(1)\n}}\n",
            undefined_variable_warning(&input, "items", 2),
            undefined_variable_warning(&input, "append", 4),
            undefined_variable_warnings(&input, &[("a", 6), ("b", 6)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_offset_compound_assignment_expressions_to_native_binary() {
    let root = temp_dir("ptn-native-array-offset-compound-expressions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-offset-compound-expressions.php");
    let output = root.join("array-offset-compound-expressions-bin");
    fs::write(
        &input,
        "<?php\n\
function idx(&$hits) {\n\
    echo \"idx:$hits\\n\";\n\
    return $hits++;\n\
}\n\
function rhs() {\n\
    echo \"rhs\\n\";\n\
    return 3;\n\
}\n\
$hits = 0;\n\
$items = [10];\n\
var_dump($items[idx($hits)] += rhs());\n\
var_dump($items[0], $hits);\n\
var_dump($append[] += 2);\n\
var_dump($append);\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "idx:0\nrhs\nint(13)\nint(13)\nint(1)\n{}int(2)\narray(1) {{\n  [0]=>\n  int(2)\n}}\n",
            undefined_variable_warning(&input, "append", 14)
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_path_read_for_assign_op"));
    assert!(c_source.contains("ptn_runtime_array_path_set_from_assign_op"));
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
fn compile_null_coalescing_variables_and_offsets_to_native_binary() {
    let root = temp_dir("ptn-native-null-coalescing");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("null-coalescing.php");
    let output = root.join("null-coalescing-bin");
    fs::write(
        &input,
        "<?php\n\
$present = \"present\";\n\
$nullish = null;\n\
$items = [\"value\" => \"hit\", \"nullish\" => null, \"nested\" => [\"leaf\" => \"\"]];\n\
$string = \"abc\";\n\
var_dump($present ?? $warn);\n\
var_dump($nullish ?? \"null-fallback\");\n\
var_dump($missing ?? \"missing-fallback\");\n\
var_dump($items[\"value\"] ?? \"array-fallback\");\n\
var_dump($items[\"nullish\"] ?? \"array-null-fallback\");\n\
var_dump($items[\"missing\"] ?? \"array-missing-fallback\");\n\
var_dump($items[\"nested\"][\"leaf\"] ?? \"nested-fallback\");\n\
var_dump($string[1] ?? \"string-fallback\");\n\
var_dump($string[99] ?? \"string-missing-fallback\");\n\
var_dump($string[\"foo\"] ?? \"string-key-fallback\");\n\
var_dump($missingArray[\"key\"] ?? \"base-fallback\");\n\
var_dump($nullish ?? $missing ?? \"chain-fallback\");",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(7) \"present\"\n\
string(13) \"null-fallback\"\n\
string(16) \"missing-fallback\"\n\
string(3) \"hit\"\n\
string(19) \"array-null-fallback\"\n\
string(22) \"array-missing-fallback\"\n\
string(0) \"\"\n\
string(1) \"b\"\n\
string(23) \"string-missing-fallback\"\n\
string(19) \"string-key-fallback\"\n\
string(13) \"base-fallback\"\n\
string(14) \"chain-fallback\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert!(main_body.contains("ptn_runtime_read_variable_quiet(&runtime"));
    assert!(main_body.contains("ptn_offset_lookup(&runtime"));
    assert!(main_body.contains(", 1);"));
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
var_dump(function_exists(\"array_key_exists\"), function_exists(\"ARRAY_KEY_EXISTS\"));\n\
class KeyCheck { public $public_var = \"Public var\"; }\n\
try { array_key_exists(array(), $items); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { array_key_exists(\"public_var\", new KeyCheck); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\nbool(false)\n\nDeprecated: Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead in ptn on line 5\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nCannot access offset of type array on array\narray_key_exists(): Argument #2 ($array) must be of type array, KeyCheck given\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_key_first_and_last_to_native_binary() {
    let root = temp_dir("ptn-native-array-key-first-last");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-key-first-last.php");
    let output = root.join("array-key-first-last-bin");
    fs::write(
        &input,
        "<?php\n\
$empty = [];\n\
var_dump(array_key_first($empty), array_key_last($empty));\n\
$items = [2 => 'a', '02' => 'b', 3 => 'c'];\n\
var_dump(array_key_first($items), array_key_last($items));\n\
$value = 'x';\n\
$refs = ['first' => &$value, 'last' => 'y'];\n\
var_dump(array_key_first($refs), array_key_last($refs));\n\
var_dump(function_exists('array_key_first'), function_exists('ARRAY_KEY_LAST'));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "NULL\nNULL\nint(2)\nint(3)\nstring(5) \"first\"\nstring(4) \"last\"\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_key_first"));
    assert!(c_source.contains("ptn_internal_array_key_last"));
}

#[test]
fn compile_array_is_list_to_native_binary() {
    let root = temp_dir("ptn-native-array-is-list");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-is-list.php");
    let output = root.join("array-is-list-bin");
    fs::write(
        &input,
        "<?php
var_dump(array_is_list([]));
var_dump(array_is_list([1, 2, 3]));
var_dump(array_is_list([1 => 'a', 0 => 'b']));
var_dump(array_is_list(['0' => 'a', 1 => 'b']));
var_dump(array_is_list(['00' => 'a', 1 => 'b']));
var_dump(array_is_list([0 => 'a', 2 => 'b']));

$drop_first = [1, 2, 3];
unset($drop_first[0]);
var_dump(array_is_list($drop_first));

$drop_middle = [1, 2, 3];
unset($drop_middle[1]);
var_dump(array_is_list($drop_middle));

$drop_end = [1, 2, 3];
unset($drop_end[2]);
var_dump(array_is_list($drop_end));

$drop_string = [1, 'a' => 'a', 2];
unset($drop_string['a']);
var_dump(array_is_list($drop_string));

$append = [1, 2, 3];
$append[] = 4;
var_dump(array_is_list($append));

$gap = [1, 2, 3];
$gap[4] = 5;
var_dump(array_is_list($gap));

try { array_is_list(null); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }
try { array_is_list(true); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }
try { array_is_list(new stdClass()); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }
var_dump(function_exists('array_is_list'), function_exists('ARRAY_IS_LIST'));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\n\
bool(true)\n\
bool(false)\n\
bool(true)\n\
bool(false)\n\
bool(false)\n\
bool(false)\n\
bool(false)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(false)\n\
array_is_list(): Argument #1 ($array) must be of type array, null given\n\
array_is_list(): Argument #1 ($array) must be of type array, true given\n\
array_is_list(): Argument #1 ($array) must be of type array, stdClass given\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_is_list"));
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
fn compile_array_keys_to_native_binary() {
    let root = temp_dir("ptn-native-array-keys");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-keys.php");
    let output = root.join("array-keys-bin");
    fs::write(
        &input,
        "<?php\n\
$items = array(\"a\" => 1, 2 => \"two\", \"03\" => null, 4 => \"2\", \"dup\" => 1);\n\
var_dump(array_keys($items));\n\
var_dump(array_keys($items, 1));\n\
var_dump(array_keys($items, \"1\", true));\n\
var_dump(array_keys($items, 2));\n\
var_dump(array_keys($items, \"2\", true));\n\
$nested = array(\"x\" => array(\"seed\"));\n\
$keys = array_keys($nested);\n\
$keys[] = \"copy\";\n\
var_dump($keys, $nested);\n\
var_dump(function_exists(\"array_keys\"), function_exists(\"ARRAY_KEYS\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(5) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  int(2)\n  [2]=>\n  string(2) \"03\"\n  [3]=>\n  int(4)\n  [4]=>\n  string(3) \"dup\"\n}\narray(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(3) \"dup\"\n}\narray(0) {\n}\narray(1) {\n  [0]=>\n  int(4)\n}\narray(1) {\n  [0]=>\n  int(4)\n}\narray(2) {\n  [0]=>\n  string(1) \"x\"\n  [1]=>\n  string(4) \"copy\"\n}\narray(1) {\n  [\"x\"]=>\n  array(1) {\n    [0]=>\n    string(4) \"seed\"\n  }\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_keys"));
}

#[test]
fn compile_array_search_to_native_binary() {
    let root = temp_dir("ptn-native-array-search");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-search.php");
    let output = root.join("array-search-bin");
    fs::write(
        &input,
        "<?php\n\
$items = array(\"a\" => 1, 2 => \"two\", \"03\" => null, 4 => \"2\", \"dup\" => 1);\n\
var_dump(array_search(1, $items));\n\
var_dump(array_search(\"1\", $items, true));\n\
var_dump(array_search(2, $items));\n\
var_dump(array_search(\"2\", $items, true));\n\
var_dump(array_search(null, $items, true));\n\
var_dump(array_search(\"missing\", $items));\n\
var_dump(array_search(\"zero\", array(\"zero\", \"one\")));\n\
var_dump(function_exists(\"array_search\"), function_exists(\"ARRAY_SEARCH\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(1) \"a\"\nbool(false)\nint(4)\nint(4)\nstring(2) \"03\"\nbool(false)\nint(0)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_search"));
}

#[test]
fn compile_array_change_key_case_to_native_binary() {
    let root = temp_dir("ptn-native-array-change-key-case");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-change-key-case.php");
    let output = root.join("array-change-key-case-bin");
    fs::write(
        &input,
        "<?php\n\
$source = array('One' => 1, 'TWO' => 2, 3 => 'three', 'two' => 4, 'MiXeD' => 'case');\n\
var_dump(array_change_key_case(array()));\n\
var_dump(array_change_key_case($source));\n\
var_dump(array_change_key_case($source, CASE_UPPER));\n\
try { array_change_key_case($source, -10); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(CASE_LOWER, CASE_UPPER, function_exists('array_change_key_case'), function_exists('ARRAY_CHANGE_KEY_CASE'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\narray(4) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(4)\n  [3]=>\n  string(5) \"three\"\n  [\"mixed\"]=>\n  string(4) \"case\"\n}\narray(4) {\n  [\"ONE\"]=>\n  int(1)\n  [\"TWO\"]=>\n  int(4)\n  [3]=>\n  string(5) \"three\"\n  [\"MIXED\"]=>\n  string(4) \"case\"\n}\narray_change_key_case(): Argument #2 ($case) must be either CASE_LOWER or CASE_UPPER\nint(0)\nint(1)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_chunk_to_native_binary() {
    let root = temp_dir("ptn-native-array-chunk");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-chunk.php");
    let output = root.join("array-chunk-bin");
    fs::write(
        &input,
        "<?php\n\
$items = array(1 => 'one','two', 3 => 'three', 4, 'five' => 5);\n\
var_dump(array_chunk($items, 2));\n\
var_dump(array_chunk($items, 2, true));\n\
$assoc = array('a' => 1, 'b' => 2, 'c' => 3);\n\
var_dump(array_chunk($assoc, 2, true));\n\
$nested = array(array('seed'), array('next'));\n\
$chunks = array_chunk($nested, 1);\n\
$chunks[0][0][] = 'copy';\n\
var_dump($chunks[0][0], $nested[0]);\n\
try { array_chunk(array(1), 0); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('array_chunk'), function_exists('ARRAY_CHUNK'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(3) {\n  [0]=>\n  array(2) {\n    [0]=>\n    string(3) \"one\"\n    [1]=>\n    string(3) \"two\"\n  }\n  [1]=>\n  array(2) {\n    [0]=>\n    string(5) \"three\"\n    [1]=>\n    int(4)\n  }\n  [2]=>\n  array(1) {\n    [0]=>\n    int(5)\n  }\n}\narray(3) {\n  [0]=>\n  array(2) {\n    [1]=>\n    string(3) \"one\"\n    [2]=>\n    string(3) \"two\"\n  }\n  [1]=>\n  array(2) {\n    [3]=>\n    string(5) \"three\"\n    [4]=>\n    int(4)\n  }\n  [2]=>\n  array(1) {\n    [\"five\"]=>\n    int(5)\n  }\n}\narray(2) {\n  [0]=>\n  array(2) {\n    [\"a\"]=>\n    int(1)\n    [\"b\"]=>\n    int(2)\n  }\n  [1]=>\n  array(1) {\n    [\"c\"]=>\n    int(3)\n  }\n}\narray(2) {\n  [0]=>\n  string(4) \"seed\"\n  [1]=>\n  string(4) \"copy\"\n}\narray(1) {\n  [0]=>\n  string(4) \"seed\"\n}\narray_chunk(): Argument #2 ($length) must be greater than 0\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_combine_preserves_reference_values_to_native_binary() {
    let root = temp_dir("ptn-native-array-combine-reference-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-combine-reference-values.php");
    let output = root.join("array-combine-reference-values-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_combine([1, \"two\"], [\"one\", 2]));\n\
var_dump(array_combine([\"x\", \"x\", false, null, 1.5], [1, 2, \"false\", \"null\", \"float\"]));\n\
$value = \"seed\";\n\
$values = [&$value];\n\
$combined = array_combine([\"ref\"], $values);\n\
$combined[\"ref\"] = \"changed\";\n\
var_dump($value, $combined);\n\
try { array_combine([1], []); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists(\"array_combine\"), function_exists(\"ARRAY_COMBINE\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [1]=>\n  string(3) \"one\"\n  [\"two\"]=>\n  int(2)\n}\narray(3) {\n  [\"x\"]=>\n  int(2)\n  [\"\"]=>\n  string(4) \"null\"\n  [\"1.5\"]=>\n  string(5) \"float\"\n}\nstring(7) \"changed\"\narray(1) {\n  [\"ref\"]=>\n  &string(7) \"changed\"\n}\narray_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_column_to_native_binary() {
    let root = temp_dir("ptn-native-array-column");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-column.php");
    let output = root.join("array-column-bin");
    fs::write(
        &input,
        "<?php\n\
$rows = [\n\
    [42 => \"a\", \"id\" => \"2\"],\n\
    [42 => \"b\", \"id\" => \"02\"],\n\
    [42 => \"c\", \"id\" => 3],\n\
    [\"x\" => \"missing\", \"id\" => \"skip\"],\n\
];\n\
var_dump(array_column($rows, 42));\n\
var_dump(array_column($rows, \"42\", \"id\"));\n\
var_dump(array_column($rows, null, \"id\"));\n\
$nested = [\"seed\"];\n\
$rows2 = [[\"key\" => \"x\", \"value\" => $nested], [\"key\" => \"y\", \"value\" => [\"next\"]]];\n\
$result = array_column($rows2, \"value\", \"key\");\n\
$result[\"x\"][] = \"copy\";\n\
var_dump($result[\"x\"], $nested);\n\
var_dump(function_exists(\"array_column\"), function_exists(\"ARRAY_COLUMN\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"c\"\n}\narray(3) {\n  [2]=>\n  string(1) \"a\"\n  [\"02\"]=>\n  string(1) \"b\"\n  [3]=>\n  string(1) \"c\"\n}\narray(4) {\n  [2]=>\n  array(2) {\n    [42]=>\n    string(1) \"a\"\n    [\"id\"]=>\n    string(1) \"2\"\n  }\n  [\"02\"]=>\n  array(2) {\n    [42]=>\n    string(1) \"b\"\n    [\"id\"]=>\n    string(2) \"02\"\n  }\n  [3]=>\n  array(2) {\n    [42]=>\n    string(1) \"c\"\n    [\"id\"]=>\n    int(3)\n  }\n  [\"skip\"]=>\n  array(2) {\n    [\"x\"]=>\n    string(7) \"missing\"\n    [\"id\"]=>\n    string(4) \"skip\"\n  }\n}\narray(2) {\n  [0]=>\n  string(4) \"seed\"\n  [1]=>\n  string(4) \"copy\"\n}\narray(1) {\n  [0]=>\n  string(4) \"seed\"\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_count_values_to_native_binary() {
    let root = temp_dir("ptn-native-array-count-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-count-values.php");
    let output = root.join("array-count-values-bin");
    fs::write(
        &input,
        "<?php\n\
$value = \"hello\";\n\
$ref =& $value;\n\
var_dump(array_count_values([]));\n\
var_dump(array_count_values([1, \"hello\", 1, \"world\", \"hello\", \"1\", -1, \"02\", \"\"]));\n\
var_dump(array_count_values([$ref, \"hello\"]));\n\
var_dump(@array_count_values([0, [1, 2], 0, true, null, 1.5]));\n\
var_dump(array_count_values([[], false, \"kept\"]));\n\
var_dump(function_exists(\"array_count_values\"), function_exists(\"ARRAY_COUNT_VALUES\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\narray(6) {\n  [1]=>\n  int(3)\n  [\"hello\"]=>\n  int(2)\n  [\"world\"]=>\n  int(1)\n  [-1]=>\n  int(1)\n  [\"02\"]=>\n  int(1)\n  [\"\"]=>\n  int(1)\n}\narray(1) {\n  [\"hello\"]=>\n  int(2)\n}\narray(1) {\n  [0]=>\n  int(2)\n}\nWarning: array_count_values(): Can only count string and integer values, entry skipped in ptn on line 8\nWarning: array_count_values(): Can only count string and integer values, entry skipped in ptn on line 8\narray(1) {\n  [\"kept\"]=>\n  int(1)\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_flip_to_native_binary() {
    let root = temp_dir("ptn-native-array-flip");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-flip.php");
    let output = root.join("array-flip-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_flip(array(1, 2)));\n\
var_dump(array_flip(array('value1', \"value2\")));\n\
var_dump(array_flip(array('key1' => 1, \"key2\" => 2)));\n\
var_dump(array_flip(array(1 => 'one', 2 => \"two\")));\n\
var_dump(array_flip(array(1 => 'one','two', 3 => 'three', 4, \"five\" => 5)));\n\
var_dump(array_flip(array('first' => 'same', 'second' => 'same', 'kept' => 'other')));\n\
var_dump(array_flip(array('ok', [], false, 'done')));\n\
var_dump(function_exists('array_flip'), function_exists('ARRAY_FLIP'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [1]=>\n  int(0)\n  [2]=>\n  int(1)\n}\narray(2) {\n  [\"value1\"]=>\n  int(0)\n  [\"value2\"]=>\n  int(1)\n}\narray(2) {\n  [1]=>\n  string(4) \"key1\"\n  [2]=>\n  string(4) \"key2\"\n}\narray(2) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n}\narray(5) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n  [\"three\"]=>\n  int(3)\n  [4]=>\n  int(4)\n  [5]=>\n  string(4) \"five\"\n}\narray(2) {\n  [\"same\"]=>\n  string(6) \"second\"\n  [\"other\"]=>\n  string(4) \"kept\"\n}\nWarning: array_flip(): Can only flip string and integer values, entry skipped in ptn on line 8\nWarning: array_flip(): Can only flip string and integer values, entry skipped in ptn on line 8\narray(2) {\n  [\"ok\"]=>\n  int(0)\n  [\"done\"]=>\n  int(3)\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_call_user_func_array_to_native_binary() {
    let root = temp_dir("ptn-native-call-user-func-array");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("call-user-func-array.php");
    let output = root.join("call-user-func-array-bin");
    fs::write(
        &input,
        "<?php\n\
function join_three($left, $middle, $right = \"C\") {\n\
    return $left . \":\" . $middle . \":\" . $right;\n\
}\n\
function add_mark(&$value) {\n\
    $value .= \"!\";\n\
}\n\
$box = [\"seed\"];\n\
var_dump(call_user_func_array(\"join_three\", [\"A\", \"B\"]));\n\
var_dump(call_user_func_array(\"strlen\", [\"abcd\"]));\n\
var_dump(call_user_func_array(\"add_mark\", [\"plain\"]));\n\
call_user_func_array(\"add_mark\", [&$box[0]]);\n\
var_dump($box[0]);\n\
var_dump(function_exists(\"call_user_func_array\"), function_exists(\"CALL_USER_FUNC_ARRAY\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(5) \"A:B:C\"\n",
            "int(4)\n",
            "\nWarning: add_mark(): Argument #1 ($value) must be passed by reference, value given in ptn on line 11\n",
            "NULL\n",
            "string(5) \"seed!\"\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_call_user_func_array"));
    assert!(c_source.contains("ptn_call_callable(runtime, args[0], arguments->len"));
}

#[test]
fn compile_error_reporting_filters_internal_warnings_to_native_binary() {
    let root = temp_dir("ptn-native-error-reporting-internal-warnings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("error-reporting-internal-warnings.php");
    let output = root.join("error-reporting-internal-warnings-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(error_reporting());\n\
var_dump(array_count_values([[], \"shown\"]));\n\
var_dump(error_reporting(E_ERROR));\n\
var_dump(error_reporting());\n\
var_dump(array_count_values([[], \"hidden\"]));\n\
var_dump(define(\"MASKED_DEFINE_FLAG\", 1, true), constant(\"MASKED_DEFINE_FLAG\"));\n\
var_dump(error_reporting(E_ALL));\n\
var_dump(error_reporting());\n\
var_dump(array_count_values([[], \"shown-again\"]));\n\
var_dump(define(\"SHOWN_DEFINE_FLAG\", 2, true), constant(\"SHOWN_DEFINE_FLAG\"));\n\
var_dump(defined(\"E_WARNING\"), defined(\"E_ALL\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(32767)\nWarning: array_count_values(): Can only count string and integer values, entry skipped in ptn on line 3\narray(1) {\n  [\"shown\"]=>\n  int(1)\n}\nint(32767)\nint(1)\narray(1) {\n  [\"hidden\"]=>\n  int(1)\n}\nbool(true)\nint(1)\nint(1)\nint(32767)\nWarning: array_count_values(): Can only count string and integer values, entry skipped in ptn on line 10\narray(1) {\n  [\"shown-again\"]=>\n  int(1)\n}\nWarning: define(): Argument #3 ($case_insensitive) is ignored since declaration of case-insensitive constants is no longer supported in ptn on line 11\nbool(true)\nint(2)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_fill_to_native_binary() {
    let root = temp_dir("ptn-native-array-fill");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-fill.php");
    let output = root.join("array-fill-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_fill(0, 0, true));\n\
var_dump(array_fill(1, 2, 'x'));\n\
$value = array('seed');\n\
$filled = array_fill(-1, 2, $value);\n\
$filled[-1][] = 'copy';\n\
var_dump($filled);\n\
try { array_fill(0, -1, 'x'); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { array_fill(PHP_INT_MAX, 2, 'x'); } catch (Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('array_fill'), function_exists('ARRAY_FILL'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\narray(2) {\n  [1]=>\n  string(1) \"x\"\n  [2]=>\n  string(1) \"x\"\n}\narray(2) {\n  [-1]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(4) \"copy\"\n  }\n  [0]=>\n  array(1) {\n    [0]=>\n    string(4) \"seed\"\n  }\n}\narray_fill(): Argument #2 ($count) must be greater than or equal to 0\nCannot add element to the array as the next element is already occupied\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_pad_to_native_binary() {
    let root = temp_dir("ptn-native-array-pad");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-pad.php");
    let output = root.join("array-pad-bin");
    fs::write(
        &input,
        "<?php\n\
$base = [\"s\" => \"S\", 4 => \"I\", \"t\" => \"T\"];\n\
var_dump(array_pad([], 1, 0));\n\
var_dump(array_pad([\"\", -1, 2.0], 5, 0));\n\
var_dump(array_pad($base, 5, \"x\"));\n\
var_dump(array_pad($base, -5, \"x\"));\n\
$nested = [\"seed\"];\n\
$padded = array_pad([\"left\" => $nested], 3, $nested);\n\
$padded[0][] = \"pad\";\n\
$padded[\"left\"][] = \"source\";\n\
var_dump($padded, $nested);\n\
var_dump(array_pad([2 => \"a\"], 1, \"x\"));\n\
try { array_pad([], PHP_INT_MAX, 0); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('array_pad'), function_exists('ARRAY_PAD'));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(1) {\n  [0]=>\n  int(0)\n}\narray(5) {\n  [0]=>\n  string(0) \"\"\n  [1]=>\n  int(-1)\n  [2]=>\n  float(2)\n  [3]=>\n  int(0)\n  [4]=>\n  int(0)\n}\narray(5) {\n  [\"s\"]=>\n  string(1) \"S\"\n  [0]=>\n  string(1) \"I\"\n  [\"t\"]=>\n  string(1) \"T\"\n  [1]=>\n  string(1) \"x\"\n  [2]=>\n  string(1) \"x\"\n}\narray(5) {\n  [0]=>\n  string(1) \"x\"\n  [1]=>\n  string(1) \"x\"\n  [\"s\"]=>\n  string(1) \"S\"\n  [2]=>\n  string(1) \"I\"\n  [\"t\"]=>\n  string(1) \"T\"\n}\narray(3) {\n  [\"left\"]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(6) \"source\"\n  }\n  [0]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(3) \"pad\"\n  }\n  [1]=>\n  array(1) {\n    [0]=>\n    string(4) \"seed\"\n  }\n}\narray(1) {\n  [0]=>\n  string(4) \"seed\"\n}\narray(1) {\n  [2]=>\n  string(1) \"a\"\n}\narray_pad(): Argument #2 ($length) must not exceed the maximum allowed array size\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_pad"));
}

#[test]
fn compile_plain_heredoc_values_to_native_binary() {
    let root = temp_dir("ptn-native-plain-heredoc-values");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("plain-heredoc-values.php");
    let output = root.join("plain-heredoc-values-bin");
    fs::write(
        &input,
        r#"<?php
$heredoc = <<<HERE_DOC
Hello
HERE_DOC;
$nowdoc = <<<'NOW_DOC'
$literal
NOW_DOC;
var_dump(strlen($heredoc), $heredoc, $nowdoc);
var_dump(array_fill(0, 2, $heredoc));
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(5)\nstring(5) \"Hello\"\nstring(8) \"$literal\"\narray(2) {\n  [0]=>\n  string(5) \"Hello\"\n  [1]=>\n  string(5) \"Hello\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_fill_keys_to_native_binary() {
    let root = temp_dir("ptn-native-array-fill-keys");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-fill-keys.php");
    let output = root.join("array-fill-keys-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_fill_keys(array(), 1));\n\
var_dump(array_fill_keys(array('foo', 'bar'), NULL));\n\
var_dump(array_fill_keys(array('5', 'foo', 10, 1.23, false, true, null, '02'), 123));\n\
$value = array('seed');\n\
$filled = array_fill_keys(array('x', 'y', 'x'), $value);\n\
$filled['x'][] = 'copy';\n\
var_dump($filled);\n\
var_dump(function_exists('array_fill_keys'), function_exists('ARRAY_FILL_KEYS'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\narray(2) {\n  [\"foo\"]=>\n  NULL\n  [\"bar\"]=>\n  NULL\n}\narray(7) {\n  [5]=>\n  int(123)\n  [\"foo\"]=>\n  int(123)\n  [10]=>\n  int(123)\n  [\"1.23\"]=>\n  int(123)\n  [\"\"]=>\n  int(123)\n  [1]=>\n  int(123)\n  [\"02\"]=>\n  int(123)\n}\narray(2) {\n  [\"x\"]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(4) \"copy\"\n  }\n  [\"y\"]=>\n  array(1) {\n    [0]=>\n    string(4) \"seed\"\n  }\n}\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_filter_to_native_binary() {
    let root = temp_dir("ptn-native-array-filter");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-filter.php");
    let output = root.join("array-filter-bin");
    fs::write(
        &input,
        "<?php\n\
function even($input) { return ($input % 2 == 0); }\n\
function key_is_keep($key) { return str_contains($key, \"keep\"); }\n\
function both_large($value, $key) { return $value > 1 && $key != \"skip\"; }\n\
$input = array(1, 2, 3, 0, -1);\n\
var_dump(array_filter($input, \"even\"));\n\
var_dump(array_filter($input));\n\
var_dump(array_filter($input, null));\n\
$assoc = array(\"keep1\" => 1, \"drop\" => 2, \"keep0\" => 0, \"skip\" => 3);\n\
var_dump(array_filter($assoc, \"key_is_keep\", ARRAY_FILTER_USE_KEY));\n\
var_dump(array_filter($assoc, \"both_large\", ARRAY_FILTER_USE_BOTH));\n\
$value = array(\"seed\");\n\
$source = array(\"x\" => $value, \"empty\" => array());\n\
$filtered = array_filter($source);\n\
$filtered[\"x\"][] = \"copy\";\n\
var_dump($filtered, $source[\"x\"]);\n\
try { array_filter($input, null, 999); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(ARRAY_FILTER_USE_BOTH, ARRAY_FILTER_USE_KEY, function_exists('array_filter'), function_exists('ARRAY_FILTER'));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [1]=>\n  int(2)\n  [3]=>\n  int(0)\n}\narray(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n  [4]=>\n  int(-1)\n}\narray(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n  [4]=>\n  int(-1)\n}\narray(2) {\n  [\"keep1\"]=>\n  int(1)\n  [\"keep0\"]=>\n  int(0)\n}\narray(1) {\n  [\"drop\"]=>\n  int(2)\n}\narray(1) {\n  [\"x\"]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(4) \"copy\"\n  }\n}\narray(1) {\n  [0]=>\n  string(4) \"seed\"\n}\narray_filter(): Argument #3 ($mode) must be one of ARRAY_FILTER_USE_VALUE, ARRAY_FILTER_USE_KEY, or ARRAY_FILTER_USE_BOTH\nint(1)\nint(2)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_filter"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, callback_argc"));
}

#[test]
fn compile_array_combine_to_native_binary() {
    let root = temp_dir("ptn-native-array-combine");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-combine.php");
    let output = root.join("array-combine-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_combine(array(), array()));\n\
var_dump(array_combine(array(1, 2), array(3, 4)));\n\
var_dump(array_combine(array(1 => 'a', 2 => 'b'), array(3 => 'c', 4 => 'd')));\n\
var_dump(array_combine(array('8', '08', 8, 1.2, false, true, null, 'x', 'x'), array('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i')));\n\
$value = array('seed');\n\
$combined = array_combine(array('first', 'second'), array($value, $value));\n\
$combined['first'][] = 'copy';\n\
var_dump($combined);\n\
try { array_combine(array(1), array(1, 2)); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('array_combine'), function_exists('ARRAY_COMBINE'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(0) {\n}\narray(2) {\n  [1]=>\n  int(3)\n  [2]=>\n  int(4)\n}\narray(2) {\n  [\"a\"]=>\n  string(1) \"c\"\n  [\"b\"]=>\n  string(1) \"d\"\n}\narray(6) {\n  [8]=>\n  string(1) \"c\"\n  [\"08\"]=>\n  string(1) \"b\"\n  [\"1.2\"]=>\n  string(1) \"d\"\n  [\"\"]=>\n  string(1) \"g\"\n  [1]=>\n  string(1) \"f\"\n  [\"x\"]=>\n  string(1) \"i\"\n}\narray(2) {\n  [\"first\"]=>\n  array(2) {\n    [0]=>\n    string(4) \"seed\"\n    [1]=>\n    string(4) \"copy\"\n  }\n  [\"second\"]=>\n  array(1) {\n    [0]=>\n    string(4) \"seed\"\n  }\n}\narray_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_range_integer_internal_to_native_binary() {
    let root = temp_dir("ptn-native-range-integer");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("range-integer.php");
    let output = root.join("range-integer-bin");
    fs::write(
        &input,
        "<?php\n\
print_r(range(1, 3));\n\
print_r(range(1, 5, 2));\n\
print_r(range(1, 3, -1));\n\
print_r(range(-1, -5, -2));\n\
print_r(range(3, 1, -1));\n\
print_r(range(3, 1, 1));\n\
print_r(range(3, 3, 9));\n\
try { range(1, 3, 0); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { range(1, 3, 5); } catch (ValueError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists('range'), function_exists('RANGE'));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Array\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\nArray\n(\n    [0] => 1\n    [1] => 3\n    [2] => 5\n)\nArray\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\nArray\n(\n    [0] => -1\n    [1] => -3\n    [2] => -5\n)\nArray\n(\n    [0] => 3\n    [1] => 2\n    [2] => 1\n)\nArray\n(\n    [0] => 3\n    [1] => 2\n    [2] => 1\n)\nArray\n(\n    [0] => 3\n)\nrange(): Argument #3 ($step) must not exceed the specified range\nrange(): Argument #3 ($step) must not exceed the specified range\nbool(true)\nbool(true)\n"
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
    assert!(c_source.contains("static PTN_UNUSED PtnValue ptn_count_value("));
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
$nested = [[1, 2, 3]];\n\
var_dump(next($nested[0]));\n\
var_dump(current($nested[0]));\n\
var_dump(end($nested[0]));\n\
var_dump(prev($nested[0]));\n\
var_dump(reset($nested[0]));\n\
$cursor_source = [[10, 20]];\n\
$cursor_copy = $cursor_source;\n\
var_dump(next($cursor_copy[0]));\n\
var_dump(current($cursor_source[0]));\n\
var_dump(current($cursor_copy[0]));\n\
var_dump(function_exists(\"ARRAY_POP\"), function_exists(\"current\"), function_exists(\"end\"), function_exists(\"prev\"), function_exists(\"reset\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(5) \"apple\"\nstring(1) \"a\"\nstring(4) \"book\"\nstring(4) \"book\"\nstring(1) \"b\"\nstring(4) \"cook\"\nstring(1) \"c\"\nstring(4) \"book\"\nstring(1) \"b\"\nstring(4) \"cook\"\nbool(false)\nbool(false)\nNULL\nstring(5) \"apple\"\nstring(1) \"a\"\nbool(false)\nNULL\nbool(false)\nbool(false)\nbool(false)\nint(3)\nint(4)\narray(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(4)\n  [3]=>\n  int(5)\n}\nstring(5) \"fubar\"\narray(2) {\n  [3]=>\n  string(3) \"foo\"\n  [4]=>\n  string(3) \"bar\"\n}\nstring(2) \"ex\"\narray(3) {\n  [0]=>\n  string(4) \"four\"\n  [1]=>\n  string(4) \"nine\"\n  [\"z\"]=>\n  string(3) \"zed\"\n}\nint(10)\narray(2) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n}\narray(2) {\n  [0]=>\n  int(30)\n  [1]=>\n  int(40)\n}\nint(2)\nint(2)\nint(3)\nint(2)\nint(1)\nint(20)\nint(10)\nint(20)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_path_cursor_and_single_mutators_to_native_binary() {
    let root = temp_dir("ptn-native-array-path-cursor-mutators");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-path-cursor-mutators.php");
    let output = root.join("array-path-cursor-mutators-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [[1, 2, 3], [\"a\", \"b\"]];\n\
var_dump(next($items[0]));\n\
var_dump(array_shift($items[0]));\n\
var_dump(current($items[0]));\n\
var_dump(array_pop($items[1]));\n\
var_dump($items);\n\
$source = [[10, 20]];\n\
$copy = $source;\n\
var_dump(array_shift($copy[0]));\n\
var_dump($source[0]);\n\
var_dump($copy[0]);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(2)\nint(1)\nint(2)\nstring(1) \"b\"\narray(2) {\n  [0]=>\n  array(2) {\n    [0]=>\n    int(2)\n    [1]=>\n    int(3)\n  }\n  [1]=>\n  array(1) {\n    [0]=>\n    string(1) \"a\"\n  }\n}\nint(10)\narray(2) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n}\narray(1) {\n  [0]=>\n  int(20)\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_next_path"));
    assert!(c_source.contains("ptn_runtime_array_shift_path"));
    assert!(c_source.contains("ptn_runtime_array_pop_path"));
}

#[test]
fn parser_rejects_temporary_array_cursor_mutation_calls() {
    for (source, function) in [
        ("<?php next([1, 2]);", "next"),
        ("<?php var_dump(reset(array(1, 2)));", "reset"),
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
    parser::parse("<?php $items = [[1], [2]]; end($items[0]);").unwrap();
}

#[test]
fn parser_rejects_non_variable_array_by_ref_mutation_calls() {
    for (source, function) in [
        ("<?php array_pop([1, 2]);", "array_pop"),
        ("<?php var_dump(array_shift(array(1, 2)));", "array_shift"),
        ("<?php array_push([1], 2);", "array_push"),
        ("<?php array_unshift([1], 2);", "array_unshift"),
        ("<?php arsort([3, 2, 1]);", "arsort"),
        ("<?php asort([3, 2, 1]);", "asort"),
        ("<?php krsort([3 => \"c\", 1 => \"a\"]);", "krsort"),
        ("<?php ksort([3 => \"c\", 1 => \"a\"]);", "ksort"),
        ("<?php natsort([\"img2\", \"img1\"]);", "natsort"),
        ("<?php rsort([3, 2, 1]);", "rsort"),
        ("<?php shuffle([1, 2, 3]);", "shuffle"),
        ("<?php sort([3, 2, 1]);", "sort"),
        ("<?php rsort([3, 2, 1]);", "rsort"),
        (
            "<?php $items = [[1], [2]]; var_dump(array_shift(current($items)));",
            "array_shift",
        ),
        (
            "<?php $items = [[1], [2]]; array_unshift($items[0], 0);",
            "array_unshift",
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
            .contains("non-variable array mutation targets are unsupported"));
    }

    parser::parse("<?php $items = [1, 2]; array_pop(($items));").unwrap();
    parser::parse("<?php $items = [1]; array_push(($items), 2);").unwrap();
    parser::parse("<?php $items = [1]; array_unshift(($items), 0);").unwrap();
    parser::parse("<?php $items = [\"b\" => 2, \"a\" => 1]; arsort(($items));").unwrap();
    parser::parse("<?php $items = [\"b\" => 2, \"a\" => 1]; asort(($items));").unwrap();
    parser::parse("<?php $items = [2 => \"b\", 1 => \"a\"]; krsort(($items));").unwrap();
    parser::parse("<?php $items = [2 => \"b\", 1 => \"a\"]; ksort(($items));").unwrap();
    parser::parse("<?php $items = [\"img2\", \"img10\", \"img1\"]; natsort(($items));").unwrap();
    parser::parse("<?php $items = [3, 2, 1]; rsort(($items));").unwrap();
    parser::parse("<?php $items = [1, 2]; shuffle(($items));").unwrap();
    parser::parse("<?php $items = [3, 2, 1]; sort(($items));").unwrap();
    parser::parse("<?php $items = [[1], [2]]; array_pop($items[0]);").unwrap();
    parser::parse("<?php $items = [[1], [2]]; array_shift($items[0]);").unwrap();
}

#[test]
fn parser_rejects_unsupported_sort_family_array_mutators() {
    for source in [
        "<?php $items = [3, 2, 1]; sort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; arsort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; asort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; ksort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; krsort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; rsort($items, SORT_REGULAR);",
        "<?php $items = [3, 2, 1]; sort($items, 0);",
        "<?php $items = [3, 2, 1]; sort($items, (SORT_REGULAR));",
    ] {
        parser::parse(source).unwrap();
    }

    for (source, function, target_message) in [
        (
            "<?php $items = [[3, 2, 1]]; asort($items[0]);",
            "asort",
            "non-variable array mutation targets are unsupported",
        ),
        (
            "<?php sort([3, 2, 1], SORT_REGULAR);",
            "sort",
            "non-variable array mutation targets are unsupported",
        ),
        (
            "<?php $items = [\"img2\", \"img1\"]; natsort($items, SORT_REGULAR);",
            "natsort",
            "extra arguments are unsupported",
        ),
        (
            "<?php $items = [\"img2\", \"img1\"]; natcasesort($items, SORT_REGULAR);",
            "natcasesort",
            "extra arguments are unsupported",
        ),
        (
            "<?php $items = [3, 2, 1]; usort($items, \"cmp\");",
            "usort",
            "sort-family array mutation semantics are unsupported",
        ),
        (
            "<?php $items = [3, 2, 1]; array_multisort($items);",
            "array_multisort",
            "sort-family array mutation semantics are unsupported",
        ),
    ] {
        let error = parser::parse(source).unwrap_err();
        assert!(
            error.message.contains(&format!("{function}()")),
            "unexpected diagnostic for {function}: {}",
            error.message
        );
        assert!(
            error.message.contains(target_message),
            "unexpected diagnostic for {function}: {}",
            error.message
        );
    }

    for (source, function) in [
        (
            "<?php $items = [3, 2, 1]; sort($items, SORT_STRING);",
            "sort",
        ),
        (
            "<?php $items = [3, 2, 1]; sort($items, sort_regular);",
            "sort",
        ),
        (
            "<?php $items = [3, 2, 1]; rsort($items, SORT_STRING);",
            "rsort",
        ),
        (
            "<?php $items = [3, 2, 1]; arsort($items, SORT_STRING);",
            "arsort",
        ),
    ] {
        let error = parser::parse(source).unwrap_err();
        assert!(error.message.contains(&format!(
            "{function}() currently supports default SORT_REGULAR"
        )));
        assert!(error.message.contains("sort flags are unsupported"));
    }
}

#[test]
fn compile_descending_sort_mutators_to_native_binary() {
    let root = temp_dir("ptn-native-descending-sort-mutators");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("descending-sort-mutators.php");
    let output = root.join("descending-sort-mutators-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => 3, \"a\" => 1, \"c\" => 2];\n\
$copy = $source;\n\
var_dump(arsort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$keys = [3 => \"c\", 1 => \"a\", 2 => \"b\"];\n\
$key_copy = $keys;\n\
var_dump(krsort($key_copy));\n\
foreach ($key_copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
$dynamic = \"arsort\";\n\
$dynamic_source = [\"low\" => 1, \"high\" => 9, \"mid\" => 5];\n\
$dynamic_copy = $dynamic_source;\n\
var_dump($dynamic($dynamic_copy));\n\
echo $dynamic_source[\"low\"], \":\", $dynamic_copy[\"high\"], \":\", key($dynamic_copy), \"\\n\";\n\
foreach ($dynamic_copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
var_dump(function_exists(\"arsort\"), function_exists(\"krsort\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "b=3\n",
            "c=2\n",
            "a=1\n",
            "3:1:2\n",
            "bool(true)\n",
            "3=c\n",
            "2=b\n",
            "1=a\n",
            "bool(true)\n",
            "1:9:high\n",
            "high=9\n",
            "mid=5\n",
            "low=1\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_arsort_variable"));
    assert!(c_source.contains("ptn_runtime_array_krsort_variable"));
    assert!(c_source.contains("ptn_array_arsort_values"));
    assert!(c_source.contains("ptn_array_krsort_entries"));
}

#[test]
fn compile_asort_mutates_direct_variable_preserves_keys_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-asort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("asort-cow.php");
    let output = root.join("asort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => 3, \"a\" => 1, \"c\" => 2];\n\
$copy = $source;\n\
var_dump(asort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$mixed = [2 => \"pear\", 7 => \"apple\", 4 => \"banana\"];\n\
asort($mixed);\n\
foreach ($mixed as $key => $word) {\n\
    echo $key, \":\", $word, \"\\n\";\n\
}\n\
var_dump(function_exists(\"asort\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "a=1\n",
            "c=2\n",
            "b=3\n",
            "3:1:2\n",
            "7:apple\n",
            "4:banana\n",
            "2:pear\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_asort_variable"));
    assert!(c_source.contains("ptn_array_asort_values"));
}

#[test]
fn compile_arsort_mutates_direct_variable_preserves_keys_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-arsort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arsort-cow.php");
    let output = root.join("arsort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => 3, \"a\" => 1, \"c\" => 2];\n\
$copy = $source;\n\
var_dump(arsort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$mixed = [2 => \"pear\", 7 => \"apple\", 4 => \"banana\"];\n\
arsort($mixed);\n\
foreach ($mixed as $key => $word) {\n\
    echo $key, \":\", $word, \"\\n\";\n\
}\n\
$dynamic = \"arsort\";\n\
$dynamic_source = [3, 1, 2];\n\
$dynamic_copy = $dynamic_source;\n\
var_dump($dynamic($dynamic_copy));\n\
foreach ($dynamic_copy as $key => $value) {\n\
    echo \"d\", $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $dynamic_source[0], \":\", $dynamic_source[1], \":\", $dynamic_source[2], \"\\n\";\n\
var_dump(function_exists(\"arsort\"), function_exists(\"ARSORT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "b=3\n",
            "c=2\n",
            "a=1\n",
            "3:1:2\n",
            "2:pear\n",
            "4:banana\n",
            "7:apple\n",
            "bool(true)\n",
            "d0=3\n",
            "d2=2\n",
            "d1=1\n",
            "3:1:2\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_arsort_variable"));
    assert!(c_source.contains("ptn_array_arsort_values"));
}

#[test]
fn compile_natsort_mutates_direct_variable_preserves_keys_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-natsort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("natsort-cow.php");
    let output = root.join("natsort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => \"file2\", \"a\" => \"file10\", \"c\" => \"file1\"];\n\
$copy = $source;\n\
var_dump(natsort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$items = [\"img12\" => \"img12\", \"img10\" => \"img10\", \"img2\" => \"img2\", \"img1\" => \"img1\"];\n\
natsort($items);\n\
foreach ($items as $key => $value) {\n\
    echo $key, \":\", $value, \"\\n\";\n\
}\n\
$leading = [\"z1\" => \"a1\", \"z01\" => \"a01\", \"z001\" => \"a001\", \"z10\" => \"a10\", \"z2\" => \"a2\", \"z02\" => \"a02\"];\n\
natsort($leading);\n\
foreach ($leading as $key => $value) {\n\
    echo $key, \"~\", $value, \"\\n\";\n\
}\n\
$dynamic = \"natsort\";\n\
$dynamic_source = [\"x2\" => \"x2\", \"x10\" => \"x10\", \"x1\" => \"x1\"];\n\
$dynamic_copy = $dynamic_source;\n\
var_dump($dynamic($dynamic_copy));\n\
foreach ($dynamic_copy as $key => $value) {\n\
    echo \"d\", $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $dynamic_source[\"x2\"], \":\", $dynamic_source[\"x10\"], \":\", $dynamic_source[\"x1\"], \"\\n\";\n\
var_dump(function_exists(\"natsort\"), function_exists(\"NATSORT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "c=file1\n",
            "b=file2\n",
            "a=file10\n",
            "file2:file10:file1\n",
            "img1:img1\n",
            "img2:img2\n",
            "img10:img10\n",
            "img12:img12\n",
            "z001~a001\n",
            "z01~a01\n",
            "z02~a02\n",
            "z1~a1\n",
            "z2~a2\n",
            "z10~a10\n",
            "bool(true)\n",
            "dx1=x1\n",
            "dx2=x2\n",
            "dx10=x10\n",
            "x2:x10:x1\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_natsort_variable"));
    assert!(c_source.contains("ptn_array_natsort_values"));
}

#[test]
fn compile_natcasesort_mutates_direct_variable_preserves_keys_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-natcasesort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("natcasesort-cow.php");
    let output = root.join("natcasesort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => \"file2\", \"a\" => \"File10\", \"c\" => \"FILE1\"];\n\
$copy = $source;\n\
var_dump(natcasesort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$items = [\"img12\" => \"IMG12\", \"img10\" => \"img10\", \"img2\" => \"Img2\", \"img1\" => \"img1\"];\n\
natcasesort($items);\n\
foreach ($items as $key => $value) {\n\
    echo $key, \":\", $value, \"\\n\";\n\
}\n\
$ties = [\"z\" => \"a\", \"y\" => \"A\", \"x\" => \"a2\", \"w\" => \"A10\", \"v\" => \"a1\"];\n\
natcasesort($ties);\n\
foreach ($ties as $key => $value) {\n\
    echo $key, \"~\", $value, \"\\n\";\n\
}\n\
$dynamic = \"natcasesort\";\n\
$dynamic_source = [\"xB2\" => \"XB2\", \"xa10\" => \"xa10\", \"xA1\" => \"xA1\"];\n\
$dynamic_copy = $dynamic_source;\n\
var_dump($dynamic($dynamic_copy));\n\
foreach ($dynamic_copy as $key => $value) {\n\
    echo \"d\", $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $dynamic_source[\"xB2\"], \":\", $dynamic_source[\"xa10\"], \":\", $dynamic_source[\"xA1\"], \"\\n\";\n\
var_dump(function_exists(\"natcasesort\"), function_exists(\"NATCASESORT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "c=FILE1\n",
            "b=file2\n",
            "a=File10\n",
            "file2:File10:FILE1\n",
            "img1:img1\n",
            "img2:Img2\n",
            "img10:img10\n",
            "img12:IMG12\n",
            "z~a\n",
            "y~A\n",
            "v~a1\n",
            "x~a2\n",
            "w~A10\n",
            "bool(true)\n",
            "dxA1=xA1\n",
            "dxa10=xa10\n",
            "dxB2=XB2\n",
            "XB2:xa10:xA1\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_natcasesort_variable"));
    assert!(c_source.contains("ptn_array_natcasesort_values"));
}

#[test]
fn compile_ksort_shuffle_and_str_shuffle_to_native_binary() {
    let root = temp_dir("ptn-native-ksort-shuffle");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("ksort-shuffle.php");
    let output = root.join("ksort-shuffle-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [3 => \"c\", 1 => \"a\", 2 => \"b\"];\n\
$copy = $items;\n\
var_dump(ksort($copy));\n\
echo key($items), \":\", key($copy), \":\", $items[3], \":\", $copy[1], \"\\n\";\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
$values = [\"a\", \"b\", \"c\", \"d\"];\n\
$values_copy = $values;\n\
var_dump(shuffle($values_copy));\n\
if (count($values) === 4 && $values[0] === \"a\" && count($values_copy) === 4 && array_key_exists(0, $values_copy) && array_key_exists(1, $values_copy) && array_key_exists(2, $values_copy) && array_key_exists(3, $values_copy) && in_array(\"a\", $values_copy, true) && in_array(\"b\", $values_copy, true) && in_array(\"c\", $values_copy, true) && in_array(\"d\", $values_copy, true)) {\n\
    echo \"shuffle-ok\\n\";\n\
} else {\n\
    echo \"shuffle-bad\\n\";\n\
}\n\
$sample = str_shuffle(\"aabb\");\n\
$a = 0;\n\
$b = 0;\n\
$other = 0;\n\
for ($i = 0; $i < strlen($sample); $i++) {\n\
    if ($sample[$i] === \"a\") {\n\
        $a++;\n\
    } elseif ($sample[$i] === \"b\") {\n\
        $b++;\n\
    } else {\n\
        $other++;\n\
    }\n\
}\n\
echo strlen($sample), \":\", $a, \":\", $b, \":\", $other, \"\\n\";\n\
var_dump(function_exists(\"ksort\"), function_exists(\"shuffle\"), function_exists(\"str_shuffle\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "3:1:c:a\n",
            "1=a\n",
            "2=b\n",
            "3=c\n",
            "bool(true)\n",
            "shuffle-ok\n",
            "4:2:2:0\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_ksort_variable"));
    assert!(c_source.contains("ptn_runtime_array_shuffle_variable"));
    assert!(c_source.contains("ptn_internal_str_shuffle"));
}

#[test]
fn compile_sort_mutates_direct_variable_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-sort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sort-cow.php");
    let output = root.join("sort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [3 => \"c\", 1 => \"a\", 2 => \"b\"];\n\
$copy = $items;\n\
var_dump(sort($copy));\n\
echo key($items), \":\", key($copy), \":\", $items[3], \":\", $copy[0], \":\", $copy[2], \"\\n\";\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
$numbers = [100, 33, 555, 22];\n\
var_dump(sort($numbers));\n\
echo $numbers[0], \":\", $numbers[1], \":\", $numbers[2], \":\", $numbers[3], \"\\n\";\n\
$dynamic = \"sort\";\n\
$source = [3, 1, 2];\n\
$dynamic_copy = $source;\n\
var_dump($dynamic($dynamic_copy));\n\
echo count($source), \":\", count($dynamic_copy), \":\", $source[0], \":\", $dynamic_copy[0], \":\", $dynamic_copy[2], \"\\n\";\n\
var_dump(function_exists(\"sort\"), function_exists(\"SORT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "3:0:c:a:c\n",
            "0=a\n",
            "1=b\n",
            "2=c\n",
            "bool(true)\n",
            "22:33:100:555\n",
            "bool(true)\n",
            "3:3:3:1:3\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_sort_variable"));
    assert!(c_source.contains("ptn_array_sort_values"));
}

#[test]
fn compile_sort_regular_flags_to_native_binary() {
    let root = temp_dir("ptn-native-sort-regular-flags");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("sort-regular-flags.php");
    let output = root.join("sort-regular-flags-bin");
    fs::write(
        &input,
        "<?php\n\
$sort = [3, 1, 2];\n\
var_dump(sort($sort, SORT_REGULAR));\n\
echo implode(\",\", $sort), \"\\n\";\n\
$asort = [\"b\" => 2, \"a\" => 1];\n\
var_dump(asort($asort, SORT_REGULAR));\n\
foreach ($asort as $key => $value) { echo $key, \"=\", $value, \"\\n\"; }\n\
$arsort = [\"b\" => 2, \"a\" => 1];\n\
var_dump(arsort($arsort, SORT_REGULAR));\n\
foreach ($arsort as $key => $value) { echo $key, \"=\", $value, \"\\n\"; }\n\
$ksort = [3 => \"c\", 1 => \"a\"];\n\
var_dump(ksort($ksort, SORT_REGULAR));\n\
foreach ($ksort as $key => $value) { echo $key, \"=\", $value, \"\\n\"; }\n\
$krsort = [3 => \"c\", 1 => \"a\"];\n\
var_dump(krsort($krsort, SORT_REGULAR));\n\
foreach ($krsort as $key => $value) { echo $key, \"=\", $value, \"\\n\"; }\n\
$rsort = [1, 3, 2];\n\
var_dump(rsort($rsort, 0));\n\
echo implode(\",\", $rsort), \"\\n\";\n\
var_dump(SORT_REGULAR, defined(\"SORT_REGULAR\"), constant(\"SORT_REGULAR\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "1,2,3\n",
            "bool(true)\n",
            "a=1\n",
            "b=2\n",
            "bool(true)\n",
            "b=2\n",
            "a=1\n",
            "bool(true)\n",
            "1=a\n",
            "3=c\n",
            "bool(true)\n",
            "3=c\n",
            "1=a\n",
            "bool(true)\n",
            "3,2,1\n",
            "int(0)\n",
            "bool(true)\n",
            "int(0)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_sort_variable"));
    assert!(c_source.contains("ptn_runtime_array_asort_variable"));
    assert!(c_source.contains("ptn_runtime_array_arsort_variable"));
    assert!(c_source.contains("ptn_runtime_array_ksort_variable"));
    assert!(c_source.contains("ptn_runtime_array_krsort_variable"));
    assert!(c_source.contains("ptn_runtime_array_rsort_variable"));
}

#[test]
fn compile_rsort_mutates_direct_variable_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-rsort-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("rsort-cow.php");
    let output = root.join("rsort-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"b\" => 3, \"a\" => 1, \"c\" => 2];\n\
$copy = $source;\n\
var_dump(rsort($copy));\n\
foreach ($copy as $key => $value) {\n\
    echo $key, \"=\", $value, \"\\n\";\n\
}\n\
echo $source[\"b\"], \":\", $source[\"a\"], \":\", $source[\"c\"], \"\\n\";\n\
$words = [\"pear\", \"apple\", \"banana\"];\n\
rsort($words);\n\
foreach ($words as $word) {\n\
    echo $word, \"\\n\";\n\
}\n\
var_dump(function_exists(\"rsort\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "0=3\n",
            "1=2\n",
            "2=1\n",
            "3:1:2\n",
            "pear\n",
            "banana\n",
            "apple\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_rsort_variable"));
    assert!(c_source.contains("ptn_array_rsort_values"));
}

#[test]
fn compile_dynamic_sort_flags_report_boundary_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-sort-flags");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-sort-flags.php");
    let output = root.join("dynamic-sort-flags-bin");
    fs::write(
        &input,
        "<?php\n\
$sortItems = [3, 2, 1];\n\
try {\n\
    call_user_func(\"sort\", $sortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
$asortItems = [\"b\" => 2, \"a\" => 1];\n\
try {\n\
    call_user_func(\"asort\", $asortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
$arsortItems = [\"b\" => 2, \"a\" => 1];\n\
try {\n\
    call_user_func(\"arsort\", $arsortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
$rsortItems = [1, 2, 3];\n\
try {\n\
    call_user_func(\"rsort\", $rsortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
$ksortItems = [3 => \"c\", 1 => \"a\"];\n\
try {\n\
    call_user_func(\"ksort\", $ksortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
$krsortItems = [3 => \"c\", 1 => \"a\"];\n\
try {\n\
    call_user_func(\"krsort\", $krsortItems, 0);\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}\n\
var_dump($sortItems, $asortItems, $arsortItems, $rsortItems, $ksortItems, $krsortItems);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "sort() flags are unsupported; default regular value sorting is supported\n",
            "asort() flags are unsupported; default regular value sorting is supported\n",
            "arsort() flags are unsupported; default regular value sorting is supported\n",
            "rsort() flags are unsupported; default regular value sorting is supported\n",
            "ksort() flags are unsupported; default regular value sorting is supported\n",
            "krsort() flags are unsupported; default regular value sorting is supported\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(3)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [2]=>\n",
            "  int(1)\n",
            "}\n",
            "array(2) {\n",
            "  [\"b\"]=>\n",
            "  int(2)\n",
            "  [\"a\"]=>\n",
            "  int(1)\n",
            "}\n",
            "array(2) {\n",
            "  [\"b\"]=>\n",
            "  int(2)\n",
            "  [\"a\"]=>\n",
            "  int(1)\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [2]=>\n",
            "  int(3)\n",
            "}\n",
            "array(2) {\n",
            "  [3]=>\n",
            "  string(1) \"c\"\n",
            "  [1]=>\n",
            "  string(1) \"a\"\n",
            "}\n",
            "array(2) {\n",
            "  [3]=>\n",
            "  string(1) \"c\"\n",
            "  [1]=>\n",
            "  string(1) \"a\"\n",
            "}\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_unshift_mutates_direct_variable_and_detaches_cow_to_native_binary() {
    let root = temp_dir("ptn-native-array-unshift-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-unshift-cow.php");
    let output = root.join("array-unshift-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"x\" => \"X\", 4 => \"A\", 9 => \"B\"];\n\
$copy = $source;\n\
var_dump(array_unshift($copy, \"first\", \"second\"));\n\
var_dump($source);\n\
var_dump($copy);\n\
$empty = [];\n\
var_dump(array_unshift($empty));\n\
var_dump($empty);\n\
function local_unshift($arr) { return array_unshift($arr, 6); }\n\
$local = [7, 8];\n\
var_dump(local_unshift($local));\n\
var_dump($local);\n\
var_dump(function_exists(\"array_unshift\"), function_exists(\"ARRAY_UNSHIFT\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "int(5)\n",
            "array(3) {\n",
            "  [\"x\"]=>\n",
            "  string(1) \"X\"\n",
            "  [4]=>\n",
            "  string(1) \"A\"\n",
            "  [9]=>\n",
            "  string(1) \"B\"\n",
            "}\n",
            "array(5) {\n",
            "  [0]=>\n",
            "  string(5) \"first\"\n",
            "  [1]=>\n",
            "  string(6) \"second\"\n",
            "  [\"x\"]=>\n",
            "  string(1) \"X\"\n",
            "  [2]=>\n",
            "  string(1) \"A\"\n",
            "  [3]=>\n",
            "  string(1) \"B\"\n",
            "}\n",
            "int(0)\n",
            "array(0) {\n",
            "}\n",
            "int(3)\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(7)\n",
            "  [1]=>\n",
            "  int(8)\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_unshift_variable"));
    assert!(c_source.contains("ptn_array_unshift_values"));
}

#[test]
fn compile_array_reverse_and_reindexing_internals_to_native_binary() {
    let root = temp_dir("ptn-native-array-reverse-reindexing-internals");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reverse-reindexing-internals.php");
    let output = root.join("array-reverse-reindexing-internals-bin");
    fs::write(
        &input,
        "<?php\n\
$refs = [\"a\", \"b\", \"c\"];\n\
foreach ($refs as &$value) {}\n\
var_dump(array_values($refs));\n\
var_dump(array_reverse($refs));\n\
$assoc = [0 => \"zero\", 1 => \"one\", 2 => \"two\", \"s\" => \"ess\", 3 => \"four\"];\n\
var_dump(array_reverse($assoc, true));\n\
var_dump(function_exists(\"array_reverse\"), function_exists(\"ARRAY_REVERSE\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(3) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  &string(1) \"c\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  &string(1) \"c\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  string(1) \"a\"\n",
            "}\n",
            "array(5) {\n",
            "  [3]=>\n",
            "  string(4) \"four\"\n",
            "  [\"s\"]=>\n",
            "  string(3) \"ess\"\n",
            "  [2]=>\n",
            "  string(3) \"two\"\n",
            "  [1]=>\n",
            "  string(3) \"one\"\n",
            "  [0]=>\n",
            "  string(4) \"zero\"\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reverse"));
    assert!(c_source.contains("ptn_array_reindexing_internal_value"));
}

#[test]
fn compile_array_slice_to_native_binary() {
    let root = temp_dir("ptn-native-array-slice");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-slice.php");
    let output = root.join("array-slice-bin");
    fs::write(
        &input,
        "<?php\n\
$source = [\"a\" => \"A\", 5 => \"five\", 6 => \"six\", \"b\" => \"B\", 7 => \"seven\"];\n\
var_dump(array_slice($source, 1, 3));\n\
var_dump(array_slice($source, 1, 3, true));\n\
var_dump(array_slice($source, -2, null, true));\n\
var_dump(array_slice($source, 1, -1));\n\
var_dump(array_slice($source, 9));\n\
$value = [\"seed\"];\n\
$nested = [$value, [\"next\"], $value];\n\
$slice = array_slice($nested, 0, 2);\n\
$slice[0][] = \"copy\";\n\
var_dump($slice[0], $nested[0]);\n\
try { array_slice(range(1, 3), 0, \"foo\"); } catch (TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(function_exists(\"array_slice\"), function_exists(\"ARRAY_SLICE\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(3) {\n",
            "  [0]=>\n",
            "  string(4) \"five\"\n",
            "  [1]=>\n",
            "  string(3) \"six\"\n",
            "  [\"b\"]=>\n",
            "  string(1) \"B\"\n",
            "}\n",
            "array(3) {\n",
            "  [5]=>\n",
            "  string(4) \"five\"\n",
            "  [6]=>\n",
            "  string(3) \"six\"\n",
            "  [\"b\"]=>\n",
            "  string(1) \"B\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"b\"]=>\n",
            "  string(1) \"B\"\n",
            "  [7]=>\n",
            "  string(5) \"seven\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(4) \"five\"\n",
            "  [1]=>\n",
            "  string(3) \"six\"\n",
            "  [\"b\"]=>\n",
            "  string(1) \"B\"\n",
            "}\n",
            "array(0) {\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  string(4) \"seed\"\n",
            "  [1]=>\n",
            "  string(4) \"copy\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(4) \"seed\"\n",
            "}\n",
            "array_slice(): Argument #3 ($length) must be of type ?int, string given\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_slice"));
    assert!(c_source.contains("ptn_array_slice_start_offset"));
}

#[test]
fn compile_array_keys_filter_edges_to_native_binary() {
    let root = temp_dir("ptn-native-array-keys-filter-edges");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-keys-filter-edges.php");
    let output = root.join("array-keys-filter-edges-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"a\" => 1, \"b\" => \"1\", 4 => false, \"05\" => null, 5 => [1], \"x\" => 1];\n\
var_dump(array_keys($items));\n\
var_dump(array_keys($items, 1));\n\
var_dump(array_keys($items, \"1\", true));\n\
var_dump(array_keys($items, null));\n\
var_dump(array_keys($items, [1], true));\n\
var_dump(function_exists(\"array_keys\"), function_exists(\"ARRAY_KEYS\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(6) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  int(4)\n",
            "  [3]=>\n",
            "  string(2) \"05\"\n",
            "  [4]=>\n",
            "  int(5)\n",
            "  [5]=>\n",
            "  string(1) \"x\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(1) \"a\"\n",
            "  [1]=>\n",
            "  string(1) \"b\"\n",
            "  [2]=>\n",
            "  string(1) \"x\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(1) \"b\"\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(4)\n",
            "  [1]=>\n",
            "  string(2) \"05\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  int(5)\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_keys"));
    assert!(c_source.contains("ptn_compare_identical"));
    assert!(c_source.contains("ptn_compare_equal"));
}

#[test]
fn compile_array_merge_to_native_binary() {
    let root = temp_dir("ptn-native-array-merge");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-merge.php");
    let output = root.join("array-merge-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_merge());\n\
$left = [10 => \"ten\", \"keep\" => \"left\", 11 => \"eleven\"];\n\
$right = [\"keep\" => \"right\", 0 => \"zero\", \"new\" => \"new\"];\n\
$merged = array_merge($left, $right);\n\
$merged[\"keep\"] = \"changed\";\n\
var_dump($merged, $left, $right, function_exists(\"array_merge\"), function_exists(\"ARRAY_MERGE\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(0) {\n",
            "}\n",
            "array(5) {\n",
            "  [0]=>\n",
            "  string(3) \"ten\"\n",
            "  [\"keep\"]=>\n",
            "  string(7) \"changed\"\n",
            "  [1]=>\n",
            "  string(6) \"eleven\"\n",
            "  [2]=>\n",
            "  string(4) \"zero\"\n",
            "  [\"new\"]=>\n",
            "  string(3) \"new\"\n",
            "}\n",
            "array(3) {\n",
            "  [10]=>\n",
            "  string(3) \"ten\"\n",
            "  [\"keep\"]=>\n",
            "  string(4) \"left\"\n",
            "  [11]=>\n",
            "  string(6) \"eleven\"\n",
            "}\n",
            "array(3) {\n",
            "  [\"keep\"]=>\n",
            "  string(5) \"right\"\n",
            "  [0]=>\n",
            "  string(4) \"zero\"\n",
            "  [\"new\"]=>\n",
            "  string(3) \"new\"\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_merge"));
    assert!(c_source.contains("ptn_array_merge_into"));
}

#[test]
fn compile_var_export_and_array_set_operations_to_native_binary() {
    let root = temp_dir("ptn-native-var-export-array-set-operations");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-export-array-set-operations.php");
    let output = root.join("var-export-array-set-operations-bin");
    fs::write(
        &input,
        "<?php\n\
$a = [0=>1, \"big\"=>2, 1=>3, 2=>6, 4=>5];\n\
$b = [2, 2, 3];\n\
$c = [-1, 1];\n\
echo var_export($a, true), \";\\n\";\n\
var_dump(array_diff($a, $b, $c));\n\
var_dump(array_diff_assoc($a, $b, $c));\n\
$intersect = [1, \"big\"=>2, 2, 6, 3, 5, 3, 454, \"some_string\", 17];\n\
$right = [2, 3, 17, \"some_string\", 7];\n\
$third = [-1, 2, 1, 15, 25, 17];\n\
var_dump(array_intersect($intersect, $right, $third));\n\
var_dump(array_intersect_assoc($intersect, $right, $third));\n\
$assoc = [\"a\"=>2, \"b\"=>\"some\", \"z\"=>\"foo\", \"f\"=>5, \"gate\"=>\"web\", 7=>18, 11=>42, 73=>\"foo\", \"som3\"=>\"some\"];\n\
$assocRight = [\"a\"=>7, 7=>18, 11=>42, \"som3\"=>\"some\", \"foo\"=>\"some\", \"goo\"=>\"foo\", \"f\"=>5, \"z\"=>\"equal\", \"gate\"=>\"web\"];\n\
$assocThird = [\"gate\"=>\"web\", 73=>\"foo\"];\n\
var_dump(array_intersect($assoc, $assocRight, $assocThird));\n\
var_dump(array_intersect_assoc($assoc, $assocRight, $assocThird));\n\
var_dump(function_exists(\"var_export\"), function_exists(\"array_diff\"), function_exists(\"ARRAY_INTERSECT_ASSOC\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array (\n",
            "  0 => 1,\n",
            "  'big' => 2,\n",
            "  1 => 3,\n",
            "  2 => 6,\n",
            "  4 => 5,\n",
            ");\n",
            "array(2) {\n",
            "  [2]=>\n",
            "  int(6)\n",
            "  [4]=>\n",
            "  int(5)\n",
            "}\n",
            "array(5) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [\"big\"]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(3)\n",
            "  [2]=>\n",
            "  int(6)\n",
            "  [4]=>\n",
            "  int(5)\n",
            "}\n",
            "array(3) {\n",
            "  [\"big\"]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [8]=>\n",
            "  int(17)\n",
            "}\n",
            "array(0) {\n",
            "}\n",
            "array(3) {\n",
            "  [\"z\"]=>\n",
            "  string(3) \"foo\"\n",
            "  [\"gate\"]=>\n",
            "  string(3) \"web\"\n",
            "  [73]=>\n",
            "  string(3) \"foo\"\n",
            "}\n",
            "array(1) {\n",
            "  [\"gate\"]=>\n",
            "  string(3) \"web\"\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_var_export"));
    assert!(c_source.contains("ptn_internal_array_diff"));
    assert!(c_source.contains("ptn_internal_array_intersect"));
}

#[test]
fn compile_var_export_embedded_nul_strings_to_native_binary() {
    let root = temp_dir("ptn-native-var-export-embedded-nul-strings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-export-embedded-nul-strings.php");
    let output = root.join("var-export-embedded-nul-strings-bin");
    fs::write(
        &input,
        "<?php\n\
$nul = chr(0);\n\
$cases = [\"\", $nul, \"a\" . $nul, $nul . \"b\", \"a\" . $nul . \"b\", \"a\" . $nul . $nul . \"b\"];\n\
echo var_export($cases, true), \"\\n\";\n\
$explodeShape = [\"a\", \"b\" . $nul . \"d\", \"f\", \"1\", \"d\"];\n\
echo md5(var_export($explodeShape, true)), \"\\n\";\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array (\n",
            "  0 => '',\n",
            "  1 => '' . \"\\0\" . '',\n",
            "  2 => 'a' . \"\\0\" . '',\n",
            "  3 => '' . \"\\0\" . 'b',\n",
            "  4 => 'a' . \"\\0\" . 'b',\n",
            "  5 => 'a' . \"\\0\" . '' . \"\\0\" . 'b',\n",
            ")\n",
            "d6bee42a771449205344c0938ad4f035\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_var_export_append_string"));
}

#[test]
fn compile_var_export_objects_to_native_binary() {
    let root = temp_dir("ptn-native-var-export-objects");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-export-objects.php");
    let output = root.join("var-export-objects-bin");
    fs::write(
        &input,
        "<?php\n\
class Cr {\n\
    private $priv_member;\n\
    public $public_member;\n\
\n\
    public function __construct($value) {\n\
        $this->priv_member = $value;\n\
        $this->public_member = $value;\n\
    }\n\
}\n\
\n\
$single = new Cr(7);\n\
$items = [\"0.1\" => new Cr(9), 0 => new Cr(23)];\n\
$std = new stdClass;\n\
$std->value = [1];\n\
\n\
echo var_export($single, true), \"\\n---\\n\";\n\
echo var_export($items, true), \"\\n---\\n\";\n\
echo var_export($std, true), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "\\Cr::__set_state(array(\n",
            "   'priv_member' => 7,\n",
            "   'public_member' => 7,\n",
            "))\n",
            "---\n",
            "array (\n",
            "  '0.1' => \n",
            "  \\Cr::__set_state(array(\n",
            "     'priv_member' => 9,\n",
            "     'public_member' => 9,\n",
            "  )),\n",
            "  0 => \n",
            "  \\Cr::__set_state(array(\n",
            "     'priv_member' => 23,\n",
            "     'public_member' => 23,\n",
            "  )),\n",
            ")\n",
            "---\n",
            "(object) array(\n",
            "   'value' => \n",
            "  array (\n",
            "    0 => 1,\n",
            "  ),\n",
            ")\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_var_export"));
    assert!(c_source.contains("__set_state"));
}

#[test]
fn compile_var_export_object_properties_with_nested_arrays_to_native_binary() {
    let root = temp_dir("ptn-native-var-export-object-property-arrays");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-export-object-property-arrays.php");
    let output = root.join("var-export-object-property-arrays-bin");
    fs::write(
        &input,
        "<?php\n\
class Cr {\n\
    private $priv_member;\n\
    public $public_member;\n\
\n\
    public function __construct($value) {\n\
        $this->priv_member = $value;\n\
        $this->public_member = [$value];\n\
    }\n\
}\n\
\n\
$items = [\"0.1\" => new Cr(9), 0 => new Cr(23)];\n\
echo var_export($items, true), \";\\n\";\n\
echo var_export(new Cr(5), true), \";\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array (\n",
            "  '0.1' => \n",
            "  \\Cr::__set_state(array(\n",
            "     'priv_member' => 9,\n",
            "     'public_member' => \n",
            "    array (\n",
            "      0 => 9,\n",
            "    ),\n",
            "  )),\n",
            "  0 => \n",
            "  \\Cr::__set_state(array(\n",
            "     'priv_member' => 23,\n",
            "     'public_member' => \n",
            "    array (\n",
            "      0 => 23,\n",
            "    ),\n",
            "  )),\n",
            ");\n",
            "\\Cr::__set_state(array(\n",
            "   'priv_member' => 5,\n",
            "   'public_member' => \n",
            "  array (\n",
            "    0 => 5,\n",
            "  ),\n",
            "));\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_var_export"));
    assert!(c_source.contains("__set_state"));
}

#[test]
fn compile_array_udiff_static_method_private_property_ternary_reducer_to_native_binary() {
    let root = temp_dir("ptn-native-array-udiff-private-property-ternary");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-udiff-private-property-ternary.php");
    let output = root.join("array-udiff-private-property-ternary-bin");
    fs::write(
        &input,
        "<?php
class Cr {
    private $priv_member;
    public $public_member;

    public function __construct($value) {
        $this->priv_member = $value;
        $this->public_member = $value;
    }

    public static function comp_func_cr($a, $b) {
        if ($a->priv_member === $b->priv_member) return 0;
        return ($a->priv_member > $b->priv_member) ? 1 : -1;
    }
}

$left = [new Cr(2), new Cr(5)];
$right = [new Cr(2)];
$result = array_udiff($left, $right, [\"Cr\", \"comp_func_cr\"]);
echo var_export($result, true), \"\\n\";
foreach ($result as $item) {
    echo $item->public_member, \"\\n\";
}
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array (\n",
            "  1 => \n",
            "  \\Cr::__set_state(array(\n",
            "     'priv_member' => 5,\n",
            "     'public_member' => 5,\n",
            "  )),\n",
            ")\n",
            "5\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_udiff_variants_to_native_binary() {
    let root = temp_dir("ptn-native-array-udiff-variants");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-udiff-variants.php");
    let output = root.join("array-udiff-variants-bin");
    fs::write(
        &input,
        "<?php\n\
class Cmp {\n\
    public static function cmp($a, $b) {\n\
        if ($a === $b) return 0;\n\
        if ($a > $b) return 1;\n\
        return -1;\n\
    }\n\
}\n\
function cmp_values($a, $b) {\n\
    if ($a === $b) return 0;\n\
    if ($a > $b) return 1;\n\
    return -1;\n\
}\n\
function cmp_keys($a, $b) {\n\
    $a = (string)$a;\n\
    $b = (string)$b;\n\
    if ($a === $b) return 0;\n\
    if ($a > $b) return 1;\n\
    return -1;\n\
}\n\
$left = [\"a\" => 3, \"b\" => 2, 0 => 5, 1 => 7, \"drop\" => 9];\n\
$right = [\"x\" => 2, 0 => 5, 1 => 8, \"drop\" => 9];\n\
var_dump(array_udiff($left, $right, [\"Cmp\", \"cmp\"]));\n\
var_dump(array_udiff_assoc($left, $right, \"cmp_values\"));\n\
var_dump(array_udiff_uassoc($left, [\"a\" => 3, \"B\" => 2, 0 => 5, 2 => 7, \"drop\" => 9], [\"Cmp\", \"cmp\"], \"cmp_keys\"));\n\
var_dump(function_exists(\"array_udiff\"), function_exists(\"ARRAY_UDIFF_ASSOC\"), function_exists(\"array_udiff_uassoc\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(2) {\n",
            "  [\"a\"]=>\n",
            "  int(3)\n",
            "  [1]=>\n",
            "  int(7)\n",
            "}\n",
            "array(3) {\n",
            "  [\"a\"]=>\n",
            "  int(3)\n",
            "  [\"b\"]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(7)\n",
            "}\n",
            "array(2) {\n",
            "  [\"b\"]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(7)\n",
            "}\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_udiff"));
    assert!(c_source.contains("ptn_internal_array_udiff_assoc"));
    assert!(c_source.contains("ptn_internal_array_udiff_uassoc"));
}

#[test]
fn compile_array_merge_recursive_to_native_binary() {
    let root = temp_dir("ptn-native-array-merge-recursive");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-merge-recursive.php");
    let output = root.join("array-merge-recursive-bin");
    fs::write(
        &input,
        "<?php\n\
var_dump(array_merge_recursive());\n\
$left = [1, [1, 2]];\n\
$right = [3, [\"hello\", \"world\"]];\n\
var_dump(array_merge_recursive($left, $right));\n\
$assoc_left = [\"k\" => \"left\", \"both\" => [\"x\" => 1]];\n\
$assoc_right = [\"k\" => \"right\", \"both\" => [\"y\" => 2]];\n\
$assoc = array_merge_recursive($assoc_left, $assoc_right);\n\
$assoc[\"both\"][\"x\"] = 9;\n\
var_dump($assoc);\n\
var_dump($assoc_left[\"both\"][\"x\"], $assoc_right[\"both\"][\"y\"], function_exists(\"array_merge_recursive\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(0) {\n",
            "}\n",
            "array(4) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "    [1]=>\n",
            "    int(2)\n",
            "  }\n",
            "  [2]=>\n",
            "  int(3)\n",
            "  [3]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    string(5) \"hello\"\n",
            "    [1]=>\n",
            "    string(5) \"world\"\n",
            "  }\n",
            "}\n",
            "array(2) {\n",
            "  [\"k\"]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    string(4) \"left\"\n",
            "    [1]=>\n",
            "    string(5) \"right\"\n",
            "  }\n",
            "  [\"both\"]=>\n",
            "  array(2) {\n",
            "    [\"x\"]=>\n",
            "    int(9)\n",
            "    [\"y\"]=>\n",
            "    int(2)\n",
            "  }\n",
            "}\n",
            "int(1)\n",
            "int(2)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_merge_recursive"));
    assert!(c_source.contains("ptn_array_merge_recursive_into"));
}

#[test]
fn compile_array_replace_recursive_to_native_binary() {
    let root = temp_dir("ptn-native-array-replace-recursive");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-replace-recursive.php");
    let output = root.join("array-replace-recursive-bin");
    fs::write(
        &input,
        "<?php\n\
$x = 24;\n\
$left = [[42], \"old\"];\n\
$right = [[&$x], \"new\"];\n\
unset($x);\n\
$merged = array_replace_recursive($left, $right);\n\
$right[0][0] = 12;\n\
$merged[0][] = \"tail\";\n\
var_dump($merged, $left, $right, function_exists(\"array_replace_recursive\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(2) {\n",
            "  [0]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(24)\n",
            "    [1]=>\n",
            "    string(4) \"tail\"\n",
            "  }\n",
            "  [1]=>\n",
            "  string(3) \"new\"\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  array(1) {\n",
            "    [0]=>\n",
            "    int(42)\n",
            "  }\n",
            "  [1]=>\n",
            "  string(3) \"old\"\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  array(1) {\n",
            "    [0]=>\n",
            "    int(12)\n",
            "  }\n",
            "  [1]=>\n",
            "  string(3) \"new\"\n",
            "}\n",
            "bool(true)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_replace_recursive"));
    assert!(c_source.contains("ptn_array_replace_recursive_into"));
}

#[test]
fn compile_array_copy_on_write_detaches_shared_payloads_to_native_binary() {
    let root = temp_dir("ptn-native-array-cow-detach");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-cow-detach.php");
    let output = root.join("array-cow-detach-bin");
    fs::write(
        &input,
        "<?php\n\
$original = [\"a\" => \"A\", \"b\" => \"B\", 2 => \"two\"];\n\
$copy = $original;\n\
$copy[\"a\"] = \"changed\";\n\
$copy[] = \"appended\";\n\
unset($copy[\"b\"]);\n\
$copy[2] = \"replaced\";\n\
var_dump($original);\n\
var_dump($copy);\n\
function mutate_array($arr) {\n\
    $arr[\"fn\"] = \"changed\";\n\
    $arr[] = \"tail\";\n\
    unset($arr[\"drop\"]);\n\
    return $arr;\n\
}\n\
$base = [\"fn\" => \"base\", \"drop\" => \"gone\"];\n\
$result = mutate_array($base);\n\
var_dump($base);\n\
var_dump($result);\n\
function identity_array($arr) { return $arr; }\n\
$returned = identity_array($base);\n\
$returned[\"fn\"] = \"return\";\n\
var_dump($base);\n\
var_dump($returned);\n\
$nested_source = [[\"x\" => 1], [\"x\" => 2]];\n\
foreach ($nested_source as $sub) {\n\
    $sub[\"x\"] = 99;\n\
    $sub[] = \"local\";\n\
}\n\
var_dump($nested_source);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(3) {\n",
            "  [\"a\"]=>\n",
            "  string(1) \"A\"\n",
            "  [\"b\"]=>\n",
            "  string(1) \"B\"\n",
            "  [2]=>\n",
            "  string(3) \"two\"\n",
            "}\n",
            "array(3) {\n",
            "  [\"a\"]=>\n",
            "  string(7) \"changed\"\n",
            "  [2]=>\n",
            "  string(8) \"replaced\"\n",
            "  [3]=>\n",
            "  string(8) \"appended\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"fn\"]=>\n",
            "  string(4) \"base\"\n",
            "  [\"drop\"]=>\n",
            "  string(4) \"gone\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"fn\"]=>\n",
            "  string(7) \"changed\"\n",
            "  [0]=>\n",
            "  string(4) \"tail\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"fn\"]=>\n",
            "  string(4) \"base\"\n",
            "  [\"drop\"]=>\n",
            "  string(4) \"gone\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"fn\"]=>\n",
            "  string(6) \"return\"\n",
            "  [\"drop\"]=>\n",
            "  string(4) \"gone\"\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  array(1) {\n",
            "    [\"x\"]=>\n",
            "    int(1)\n",
            "  }\n",
            "  [1]=>\n",
            "  array(1) {\n",
            "    [\"x\"]=>\n",
            "    int(2)\n",
            "  }\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("size_t refcount;"));
    assert!(c_source.contains("ptn_array_retain(value.as.array);"));
    assert!(c_source.contains("if (array->refcount > 1)"));
    assert!(c_source.contains("ptn_array_detach_value(&runtime->symbols.items[index].value);"));
    assert!(c_source.contains("ptn_runtime_array_detach_variable"));
}

#[test]
fn compile_compound_shared_writes_detach_to_native_binary() {
    let root = temp_dir("ptn-native-compound-shared-write-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("compound-shared-write-cow.php");
    let output = root.join("compound-shared-write-cow-bin");
    fs::write(
        &input,
        "<?php\n\
function note($id, $ok) {\n\
    echo $id;\n\
    if ($ok) {\n\
        echo \" pass\\n\";\n\
    } else {\n\
        echo \" fail\\n\";\n\
    }\n\
}\n\
$original = [\"n\" => 1, \"text\" => \"a\", \"drop\" => \"gone\"];\n\
$copy = $original;\n\
$copy[\"n\"] += 4;\n\
note(1, $original[\"n\"] === 1 && $copy[\"n\"] === 5);\n\
$copy[\"text\"] .= \"b\";\n\
note(2, $original[\"text\"] === \"a\" && $copy[\"text\"] === \"ab\");\n\
$copy[] .= \"tail\";\n\
note(3, count($original) === 3 && $copy[0] === \"tail\");\n\
unset($copy[\"drop\"]);\n\
note(4, array_key_exists(\"drop\", $original) && !array_key_exists(\"drop\", $copy));\n\
$nested = [[\"x\" => 10, \"s\" => \"q\", \"u\" => 1]];\n\
$nested_copy = $nested;\n\
$nested_copy[0][\"x\"] += 5;\n\
note(5, $nested[0][\"x\"] === 10 && $nested_copy[0][\"x\"] === 15);\n\
$nested_copy[0][\"s\"] .= \"r\";\n\
note(6, $nested[0][\"s\"] === \"q\" && $nested_copy[0][\"s\"] === \"qr\");\n\
$nested_copy[0][] = \"new\";\n\
note(7, count($nested[0]) === 3 && $nested_copy[0][0] === \"new\");\n\
unset($nested_copy[0][\"u\"]);\n\
note(8, array_key_exists(\"u\", $nested[0]) && !array_key_exists(\"u\", $nested_copy[0]));\n\
$union_source = [[\"left\" => \"source\"]];\n\
$union_copy = $union_source;\n\
$union_copy[0] += $union_copy;\n\
note(9, count($union_source[0]) === 1 && count($union_copy[0]) === 2 && $union_copy[0][\"left\"] === \"source\" && is_array($union_copy[0][0]) && $union_copy[0][0][\"left\"] === \"source\");\n\
$union_copy[0][\"left\"] = \"changed\";\n\
note(10, $union_source[0][\"left\"] === \"source\" && $union_copy[0][\"left\"] === \"changed\" && $union_copy[0][0][\"left\"] === \"source\");\n\
$str = \"abcd\";\n\
$str_copy = $str;\n\
$str_copy[1] = \"Z\";\n\
note(11, $str === \"abcd\" && $str_copy === \"aZcd\");\n\
$cat = \"left\";\n\
$cat_copy = $cat;\n\
$cat_copy .= \"-right\";\n\
note(12, $cat === \"left\" && $cat_copy === \"left-right\");",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "1 pass\n",
            "2 pass\n",
            "3 pass\n",
            "4 pass\n",
            "5 pass\n",
            "6 pass\n",
            "7 pass\n",
            "8 pass\n",
            "9 pass\n",
            "10 pass\n",
            "11 pass\n",
            "12 pass\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_path_read_for_assign_op"));
    assert!(c_source.contains("ptn_runtime_array_path_set_from_assign_op"));
    assert!(c_source.contains("ptn_runtime_array_path_unset"));
    assert!(c_source.contains("ptn_array_union"));
    assert!(c_source.contains("ptn_array_detach_value(value);"));
    assert!(c_source.contains("ptn_array_detach_value(entry_value);"));
    assert!(c_source.contains("ptn_value_detach_for_write"));
    assert!(c_source.contains("ptn_string_value_resize"));
}

#[test]
fn compile_array_copy_on_write_detaches_cursor_mutating_internals_to_native_binary() {
    let root = temp_dir("ptn-native-array-cow-cursor-internals");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-cow-cursor-internals.php");
    let output = root.join("array-cow-cursor-internals-bin");
    fs::write(
        &input,
        "<?php\n\
$cursor = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
$cursor_copy = $cursor;\n\
var_dump(next($cursor_copy));\n\
var_dump(key($cursor_copy));\n\
var_dump(current($cursor));\n\
var_dump(key($cursor));\n\
var_dump(reset($cursor_copy));\n\
var_dump(key($cursor_copy));\n\
var_dump(key($cursor));\n\
$end_copy = $cursor;\n\
var_dump(end($end_copy));\n\
var_dump(key($end_copy));\n\
var_dump(key($cursor));\n\
var_dump(prev($end_copy));\n\
var_dump(key($end_copy));\n\
var_dump(key($cursor));\n\
$numbers = [1, 2, 3];\n\
$numbers_copy = $numbers;\n\
var_dump(array_pop($numbers_copy));\n\
var_dump(array_push($numbers_copy, 4));\n\
var_dump(array_shift($numbers_copy));\n\
var_dump($numbers);\n\
var_dump($numbers_copy);\n\
function local_shift($arr) {\n\
    var_dump(array_shift($arr));\n\
    return $arr;\n\
}\n\
$base = [10, 20, 30];\n\
$shifted = local_shift($base);\n\
var_dump($base);\n\
var_dump($shifted);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(1) \"B\"\n",
            "string(1) \"b\"\n",
            "string(1) \"A\"\n",
            "string(1) \"a\"\n",
            "string(1) \"A\"\n",
            "string(1) \"a\"\n",
            "string(1) \"a\"\n",
            "string(1) \"C\"\n",
            "string(1) \"c\"\n",
            "string(1) \"a\"\n",
            "string(1) \"B\"\n",
            "string(1) \"b\"\n",
            "string(1) \"a\"\n",
            "int(3)\n",
            "int(3)\n",
            "int(1)\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [2]=>\n",
            "  int(3)\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(4)\n",
            "}\n",
            "int(10)\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(10)\n",
            "  [1]=>\n",
            "  int(20)\n",
            "  [2]=>\n",
            "  int(30)\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(20)\n",
            "  [1]=>\n",
            "  int(30)\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_next_variable"));
    assert!(c_source.contains("ptn_runtime_array_end_variable"));
    assert!(c_source.contains("ptn_runtime_array_prev_variable"));
    assert!(c_source.contains("ptn_runtime_array_reset_variable"));
    assert!(c_source.contains("ptn_runtime_array_pop_variable"));
    assert!(c_source.contains("ptn_runtime_array_push_variable"));
    assert!(c_source.contains("ptn_runtime_array_shift_variable"));
}

#[test]
fn compile_mutating_internal_cow_matrix_to_native_binary() {
    let root = temp_dir("ptn-native-mutating-internal-cow-matrix");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("mutating-internal-cow-matrix.php");
    let output = root.join("mutating-internal-cow-matrix-bin");
    fs::write(
        &input,
        "<?php\n\
function cow_local_pop($arr) {\n\
    return array_pop($arr);\n\
}\n\
function cow_local_push($arr) {\n\
    return array_push($arr, 9);\n\
}\n\
function cow_local_unshift($arr) {\n\
    return array_unshift($arr, 6);\n\
}\n\
$pass = 0;\n\
$fail = 0;\n\
$pop_source = [1, 2, 3];\n\
$pop_copy = $pop_source;\n\
$pop_value = array_pop($pop_copy);\n\
if ($pop_value === 3 && count($pop_source) === 3 && count($pop_copy) === 2 && $pop_source[2] === 3) { $pass++; } else { echo \"FAIL array_pop\\n\"; $fail++; }\n\
$push_source = [1, 2];\n\
$push_copy = $push_source;\n\
$push_count = array_push($push_copy, 3);\n\
if ($push_count === 3 && count($push_source) === 2 && count($push_copy) === 3 && $push_copy[2] === 3) { $pass++; } else { echo \"FAIL array_push\\n\"; $fail++; }\n\
$push_many_source = [1];\n\
$push_many_copy = $push_many_source;\n\
$push_many_count = array_push($push_many_copy, 2, 3);\n\
if ($push_many_count === 3 && count($push_many_source) === 1 && count($push_many_copy) === 3 && $push_many_copy[2] === 3) { $pass++; } else { echo \"FAIL array_push_many\\n\"; $fail++; }\n\
$unshift_source = [\"x\" => \"X\", 4 => \"A\", 9 => \"B\"];\n\
$unshift_copy = $unshift_source;\n\
$unshift_count = array_unshift($unshift_copy, \"first\", \"second\");\n\
if ($unshift_count === 5 && count($unshift_source) === 3 && count($unshift_copy) === 5 && $unshift_copy[0] === \"first\" && $unshift_copy[1] === \"second\" && $unshift_copy[\"x\"] === \"X\" && $unshift_copy[2] === \"A\" && $unshift_copy[3] === \"B\" && $unshift_source[4] === \"A\") { $pass++; } else { echo \"FAIL array_unshift\\n\"; $fail++; }\n\
$shift_source = [10, 20, 30];\n\
$shift_copy = $shift_source;\n\
$shift_value = array_shift($shift_copy);\n\
if ($shift_value === 10 && count($shift_source) === 3 && count($shift_copy) === 2 && $shift_source[0] === 10 && $shift_copy[0] === 20) { $pass++; } else { echo \"FAIL array_shift\\n\"; $fail++; }\n\
$next_source = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
$next_copy = $next_source;\n\
$next_value = next($next_copy);\n\
if ($next_value === \"B\" && key($next_copy) === \"b\" && key($next_source) === \"a\") { $pass++; } else { echo \"FAIL next\\n\"; $fail++; }\n\
$end_source = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
$end_copy = $end_source;\n\
$end_value = end($end_copy);\n\
if ($end_value === \"C\" && key($end_copy) === \"c\" && key($end_source) === \"a\") { $pass++; } else { echo \"FAIL end\\n\"; $fail++; }\n\
$prev_source = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
$prev_copy = $prev_source;\n\
end($prev_copy);\n\
$prev_value = prev($prev_copy);\n\
if ($prev_value === \"B\" && key($prev_copy) === \"b\" && key($prev_source) === \"a\") { $pass++; } else { echo \"FAIL prev\\n\"; $fail++; }\n\
$reset_source = [\"a\" => \"A\", \"b\" => \"B\", \"c\" => \"C\"];\n\
next($reset_source);\n\
$reset_copy = $reset_source;\n\
$reset_value = reset($reset_copy);\n\
if ($reset_value === \"A\" && key($reset_copy) === \"a\" && key($reset_source) === \"b\") { $pass++; } else { echo \"FAIL reset\\n\"; $fail++; }\n\
$string_source = \"abcd\";\n\
$string_copy = $string_source;\n\
$string_copy[1] = \"X\";\n\
if ($string_source === \"abcd\" && $string_copy === \"aXcd\") { $pass++; } else { echo \"FAIL string_offset_write\\n\"; $fail++; }\n\
$extend_source = \"ab\";\n\
$extend_copy = $extend_source;\n\
$extend_copy[4] = \"Z\";\n\
if ($extend_source === \"ab\" && $extend_copy === \"ab  Z\") { $pass++; } else { echo \"FAIL string_offset_extend\\n\"; $fail++; }\n\
$local_pop_source = [7, 8];\n\
$local_pop_value = cow_local_pop($local_pop_source);\n\
if ($local_pop_value === 8 && count($local_pop_source) === 2 && $local_pop_source[1] === 8) { $pass++; } else { echo \"FAIL local_array_pop\\n\"; $fail++; }\n\
$local_push_source = [7, 8];\n\
$local_push_count = cow_local_push($local_push_source);\n\
if ($local_push_count === 3 && count($local_push_source) === 2 && $local_push_source[1] === 8) { $pass++; } else { echo \"FAIL local_array_push\\n\"; $fail++; }\n\
$local_unshift_source = [7, 8];\n\
$local_unshift_count = cow_local_unshift($local_unshift_source);\n\
if ($local_unshift_count === 3 && count($local_unshift_source) === 2 && $local_unshift_source[0] === 7) { $pass++; } else { echo \"FAIL local_array_unshift\\n\"; $fail++; }\n\
echo \"mutating internal COW matrix: pass=\", $pass, \" fail=\", $fail, \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "mutating internal COW matrix: pass=14 fail=0\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_pop_variable"));
    assert!(c_source.contains("ptn_runtime_array_push_variable"));
    assert!(c_source.contains("ptn_runtime_array_shift_variable"));
    assert!(c_source.contains("ptn_runtime_array_unshift_variable"));
    assert!(c_source.contains("ptn_runtime_array_next_variable"));
    assert!(c_source.contains("ptn_runtime_array_end_variable"));
    assert!(c_source.contains("ptn_runtime_array_prev_variable"));
    assert!(c_source.contains("ptn_runtime_array_reset_variable"));
    assert!(c_source.contains("ptn_runtime_string_offset_set"));
}

#[test]
fn compile_cow_debug_counters_assert_repeated_array_cycles_to_native_binary() {
    let root = temp_dir("ptn-native-cow-debug-array-cycles");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("cow-debug-array-cycles.php");
    let output = root.join("cow-debug-array-cycles-bin");
    fs::write(
        &input,
        "<?php\n\
_ptn_cow_debug_reset();\n\
for ($i = 0; $i < 6; $i++) {\n\
    $source = [\"value\" => $i, \"drop\" => \"x\"];\n\
    $copy = $source;\n\
    $copy[\"value\"] = $i + 10;\n\
    $copy[] = \"tail\";\n\
    unset($copy[\"drop\"]);\n\
    unset($copy, $source);\n\
}\n\
_ptn_cow_debug_assert_counter(\"array.detach\", 6);\n\
_ptn_cow_debug_assert_counter(\"array.clone\", 6);\n\
_ptn_cow_debug_assert_counter(\"array.retain\", 12);\n\
_ptn_cow_debug_assert_counter(\"array.release\", 24);\n\
_ptn_cow_debug_assert_counter(\"array.live\", 0);\n\
_ptn_cow_debug_assert_balanced();\n\
echo _ptn_cow_debug_counter(\"array.detach\"), \":\", _ptn_cow_debug_counter(\"array.retain\"), \":\", _ptn_cow_debug_counter(\"array.release\"), \":\", _ptn_cow_debug_counter(\"array.live\"), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "6:12:24:0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("PtnCowDebugCounters"));
    assert!(c_source.contains("ptn_cow_debug_note_array_detach();"));
    assert!(c_source.contains("ptn_cow_debug_assert_balanced();"));
}

#[test]
fn compile_recursive_array_literal_cycles_are_collected_to_native_binary() {
    let root = temp_dir("ptn-native-recursive-array-literal-cycle-cleanup");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("recursive-array-literal-cycle-cleanup.php");
    let output = root.join("recursive-array-literal-cycle-cleanup-bin");
    fs::write(
        &input,
        "<?php\n\
_ptn_cow_debug_reset();\n\
for ($i = 0; $i < 6; $i++) {\n\
    $array = [&$array];\n\
    unset($array);\n\
}\n\
_ptn_cow_debug_assert_counter(\"array.live\", 0);\n\
_ptn_cow_debug_assert_balanced();\n\
echo _ptn_cow_debug_counter(\"array.live\"), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_array_break_reference_cycle"));
}

#[test]
fn compile_cow_debug_counters_assert_repeated_string_cycles_to_native_binary() {
    let root = temp_dir("ptn-native-cow-debug-string-cycles");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("cow-debug-string-cycles.php");
    let output = root.join("cow-debug-string-cycles-bin");
    fs::write(
        &input,
        "<?php\n\
_ptn_cow_debug_reset();\n\
for ($i = 0; $i < 6; $i++) {\n\
    $text = \"abcd\";\n\
    $copy = $text;\n\
    $copy[1] = \"Z\";\n\
    $copy[3] = \"Q\";\n\
    unset($copy, $text);\n\
}\n\
_ptn_cow_debug_assert_counter(\"string.detach\", 12);\n\
_ptn_cow_debug_assert_counter(\"string.retain\", 6);\n\
_ptn_cow_debug_assert_counter(\"string.release\", 18);\n\
_ptn_cow_debug_assert_counter(\"string.live\", 0);\n\
_ptn_cow_debug_assert_balanced();\n\
echo _ptn_cow_debug_counter(\"string.detach\"), \":\", _ptn_cow_debug_counter(\"string.retain\"), \":\", _ptn_cow_debug_counter(\"string.release\"), \":\", _ptn_cow_debug_counter(\"string.live\"), \"\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "12:6:18:0\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_cow_debug_note_string_detach();"));
    assert!(c_source.contains("ptn_cow_debug_note_string_retain();"));
    assert!(c_source.contains("ptn_cow_debug_note_string_release();"));
    assert!(c_source.contains("ptn_cow_debug_note_string_free();"));
    assert!(c_source.contains("_ptn_cow_debug_assert_counter"));
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
fn compile_string_assignment_alias_detaches_on_offset_write_to_native_binary() {
    let root = temp_dir("ptn-native-string-assignment-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-assignment-cow.php");
    let output = root.join("string-assignment-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$left = \"abcd\";\n\
$right = $left;\n\
$left[1] = \"X\";\n\
var_dump($left);\n\
var_dump($right);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"aXcd\"\nstring(4) \"abcd\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_parameter_alias_detaches_on_offset_write_to_native_binary() {
    let root = temp_dir("ptn-native-string-parameter-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-parameter-cow.php");
    let output = root.join("string-parameter-cow-bin");
    fs::write(
        &input,
        "<?php\n\
function mutate($value) {\n\
    $value[0] = \"Z\";\n\
    var_dump($value);\n\
}\n\
$source = \"abcd\";\n\
mutate($source);\n\
var_dump($source);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"Zbcd\"\nstring(4) \"abcd\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_string_concat_assignment_aliases_keep_original_payload_to_native_binary() {
    let root = temp_dir("ptn-native-string-concat-assign-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-concat-assign-cow.php");
    let output = root.join("string-concat-assign-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$text = \"base\";\n\
$alias = $text;\n\
$text .= \"-tail\";\n\
var_dump($text);\n\
var_dump($alias);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(9) \"base-tail\"\nstring(4) \"base\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_binary_nul_string_alias_detaches_and_preserves_length_to_native_binary() {
    let root = temp_dir("ptn-native-string-binary-nul-cow");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-binary-nul-cow.php");
    let output = root.join("string-binary-nul-cow-bin");
    fs::write(
        &input,
        "<?php\n\
$source = \"A\" . chr(0) . \"C\";\n\
$alias = $source;\n\
$source[1] = \"B\";\n\
echo strlen($source), \":\", bin2hex($source), \"\\n\";\n\
echo strlen($alias), \":\", bin2hex($alias), \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "3:414243\n3:410043\n"
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
fn compile_string_offset_assign_op_diagnostics_to_native_binary() {
    let root = temp_dir("ptn-native-string-offset-assign-op-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("string-offset-assign-op-diagnostics.php");
    let output = root.join("string-offset-assign-op-diagnostics-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"abcd\";\n\
try { $str[1] += 1; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $str[\"1str\"] += 1; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $str[\"foo\"] += 1; } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $str[1][\"y\"] += 1; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $str[\"2x\"][\"y\"] += 1; } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $str[\"1.5\"][\"y\"] += 1; } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump($str);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot use assign-op operators with string offsets\n\nWarning: Illegal string offset \"1str\" in ptn on line 4\nCannot use assign-op operators with string offsets\nCannot access offset of type string on string\nCannot use string offset as an array\n\nWarning: Illegal string offset \"2x\" in ptn on line 7\nCannot use string offset as an array\nCannot access offset of type string on string\nstring(4) \"abcd\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_nested_string_offset_unset_errors_to_native_binary() {
    let root = temp_dir("ptn-native-nested-string-offset-unset-errors");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-string-offset-unset-errors.php");
    let output = root.join("nested-string-offset-unset-errors-bin");
    fs::write(
        &input,
        "<?php
$text = \"wxyz\";
$items = [\"plain\" => \"abcd\", \"ref\" => &$text];
try { unset($items[\"plain\"][1]); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }
try { unset($items[\"ref\"][2]); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }
var_dump($items[\"plain\"]);
var_dump($items[\"ref\"]);
var_dump($text);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Cannot unset string offsets\nCannot unset string offsets\nstring(4) \"abcd\"\nstring(4) \"wxyz\"\nstring(4) \"wxyz\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_path_unset"));
    assert!(c_source.contains("Cannot unset string offsets"));
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
fn compile_bitwise_shift_variation_str2_phpt_rows_to_native_binary() {
    let root = temp_dir("ptn-native-bitwise-shift-variation-str2");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-shift-variation-str2.php");
    let output = root.join("bitwise-shift-variation-str2-bin");
    fs::write(
        &input,
        "<?php error_reporting(E_ERROR); var_dump(\"12\" << \"0\"); var_dump(\"34\" << \"1\"); var_dump(\"56\" << \"2\"); var_dump(\"12\" >> \"0\"); var_dump(\"34\" >> \"1\"); var_dump(\"56\" >> \"2\"); var_dump(defined(\"E_ERROR\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(12)\nint(68)\nint(224)\nint(12)\nint(17)\nint(14)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let left_body =
        generated_c_static_function_body(&c_source, "static PTN_UNUSED PtnValue ptn_shift_left(");
    assert!(left_body.contains("ptn_bitwise_integer_operand(left)"));
    assert!(left_body.contains("ptn_shift_distance(right)"));
    let right_body =
        generated_c_static_function_body(&c_source, "static PTN_UNUSED PtnValue ptn_shift_right(");
    assert!(right_body.contains("ptn_bitwise_integer_operand(left)"));
    assert!(right_body.contains("ptn_shift_distance(right)"));
    assert!(c_source.contains(" = ptn_shift_left("));
    assert!(c_source.contains(" = ptn_shift_right("));
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
fn compile_pow_internal_function_to_native_binary() {
    let root = temp_dir("ptn-native-pow-internal-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("pow-internal-function.php");
    let output = root.join("pow-internal-function-bin");
    fs::write(
        &input,
        "<?php var_dump(pow(2, 24), pow(2, -1), pow(\"3\", \"2\"), function_exists(\"pow\"), function_exists(\"POW\"));",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(16777216)\nfloat(0.5)\nint(9)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_pow"));
    assert!(c_source.contains("return ptn_power(runtime, args[0], args[1], line);"));
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
fn phpc_precision_ini_controls_scalar_float_stringification() {
    let root = temp_dir("ptn-phpc-precision-ini");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("precision-ini.php");
    fs::write(
        &input,
        "<?php echo strlen(10.55555555555555555555555555), \" \", strlen(10.55555555595555555555555555), \" \", 10.55555555555555555555555555, \" \", 10.55555555595555555555555555, \"\\n\";",
    )
    .unwrap();

    let execution = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .arg("-d")
        .arg("precision=12")
        .arg("-f")
        .arg(&input)
        .output()
        .unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "13 12 10.5555555556 10.555555556\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn phpc_error_reporting_ini_sets_initial_level() {
    let root = temp_dir("ptn-phpc-error-reporting-ini");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("error-reporting-ini.php");
    fs::write(&input, "<?php var_dump(error_reporting());").unwrap();

    let execution = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .arg("-d")
        .arg("error_reporting=8192")
        .arg("-f")
        .arg(&input)
        .output()
        .unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "int(8192)\n");
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}fallback\n",
            undefined_variable_warning(&input, "missing", 1)
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_branch_condition_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-branch-condition-assignments");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("branch-condition-assignments.php");
    let output = root.join("branch-condition-assignments-bin");
    fs::write(
        &input,
        "<?php if ($a = \"0\") { echo \"bad\\n\"; } else { echo \"if:$a\\n\"; } if ($b = \"ok\") { echo \"if:$b\\n\"; } $i = 0; while ($i += 1) { echo \"while:$i\\n\"; if ($i >= 2) { $i = -1; } } $j = 0; for (; $j += 1; ) { echo \"for:$j\\n\"; if ($j >= 2) { $j = -1; } } if ($cond += $missing) { echo \"bad\\n\"; } else { echo \"compound-false:$cond\\n\"; }",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "if:0\nif:ok\nwhile:1\nwhile:2\nfor:1\nfor:2\n{}compound-false:0\n",
            undefined_variable_warnings(&input, &[("cond", 1), ("missing", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn compile_include_return_value_and_output_to_native_binary() {
    let root = temp_dir("ptn-native-include-return-output");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("main.php");
    let returned = root.join("returned.php");
    let plain = root.join("plain.php");
    let output = root.join("include-return-output-bin");
    fs::write(
        &returned,
        "<?php echo \"output:$prefix\\n\"; $after = \"set\"; return \"returned\"; echo \"never\\n\";",
    )
    .unwrap();
    fs::write(&plain, "<?php echo \"plain-output\\n\";").unwrap();
    fs::write(
        &input,
        "<?php $prefix = \"scope\"; $value = include \"returned.php\"; echo \"value=$value after=$after\\n\"; $plain = include (__DIR__ . \"/plain.php\"); var_dump($plain);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "output:scope\nvalue=returned after=set\nplain-output\nint(1)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_bounded_dynamic_include_paths_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-include-paths");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("main.php");
    let base = root.join("base.php");
    let alt = root.join("alt.php");
    let output = root.join("dynamic-include-paths-bin");
    fs::write(
        &base,
        "<?php echo \"base:$outer\\n\"; $seen = \"base\"; return \"B\";",
    )
    .unwrap();
    fs::write(
        &alt,
        "<?php echo \"alt:$outer\\n\"; $seen = \"alt\"; return \"A\";",
    )
    .unwrap();
    fs::write(
        &input,
        "<?php $outer = \"scope\"; $use_alt = false; $value = include (__DIR__ . ($use_alt ? \"/alt.php\" : \"/base.php\")); echo \"value=$value seen=$seen\\n\"; $use_alt = true; $value = require (__DIR__ . ($use_alt ? \"/alt.php\" : \"/base.php\")); echo \"value=$value seen=$seen\\n\";",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "base:scope\nvalue=B seen=base\nalt:scope\nvalue=A seen=alt\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_include_resolve_path"));
    assert!(c_source.contains("ptn_include_file_0(&runtime)"));
    assert!(c_source.contains("ptn_include_file_1(&runtime)"));
}

#[test]
fn compile_include_once_and_require_once_to_native_binary() {
    let root = temp_dir("ptn-native-include-once");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("main.php");
    let once = root.join("once.php");
    let output = root.join("include-once-bin");
    fs::write(
        &once,
        "<?php echo \"once:$label\\n\"; $runs += 1; return \"RET\";",
    )
    .unwrap();
    fs::write(
        &input,
        "<?php\n\
$label = \"first\";\n\
$runs = 0;\n\
$a = include_once \"once.php\";\n\
$label = \"second\";\n\
$b = include_once (__DIR__ . \"/once.php\");\n\
$c = require_once \"once.php\";\n\
echo \"values=$a/$b/$c runs=$runs\\n\";\n\
$plain = include \"once.php\";\n\
echo \"plain=$plain runs=$runs\\n\";\n\
$d = include_once \"once.php\";\n\
var_dump($d);\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "once:first\nvalues=RET/1/1 runs=1\nonce:second\nplain=RET runs=2\nbool(true)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("static PTN_UNUSED unsigned char ptn_include_seen[1] = {0};"));
    assert!(c_source.contains("ptn_include_seen[0]"));
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!("{}done\n", undefined_variable_warning(&input, "missing", 1))
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        undefined_variable_warning(&input, "x", 2)
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
fn compile_switch_uses_loose_object_property_equality_to_native_binary() {
    let root = temp_dir("ptn-native-switch-object-loose-equality");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("switch-object-loose-equality.php");
    let output = root.join("switch-object-loose-equality-bin");
    fs::write(
        &input,
        "<?php\n\
$subject = new stdClass;\n\
$subject->name = \"ptn\";\n\
$subject->count = 2;\n\
$case = new stdClass;\n\
$case->count = \"2\";\n\
$case->name = \"ptn\";\n\
switch ($subject) {\n\
    case $case:\n\
        echo \"object-match\\n\";\n\
        break;\n\
    default:\n\
        echo \"bad\\n\";\n\
}\n\
$other = new stdClass;\n\
$other->name = \"ptn\";\n\
$other->count = 3;\n\
switch ($subject) {\n\
    case $other:\n\
        echo \"bad\\n\";\n\
        break;\n\
    default:\n\
        echo \"object-miss\\n\";\n\
}\n\
var_dump($subject == $case, $subject === $case);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "object-match\nobject-miss\nbool(true)\nbool(false)\n"
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
fn compile_array_offset_increment_decrement_statements_to_native_binary() {
    let root = temp_dir("ptn-native-array-offset-inc-dec-statements");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-offset-inc-dec-statements.php");
    let output = root.join("array-offset-inc-dec-statements-bin");
    fs::write(
        &input,
        "<?php\n\
$counts = [\"a\" => 0, \"b\" => 2];\n\
$key = \"a\";\n\
$counts[$key]++;\n\
++$counts[\"a\"];\n\
$counts[\"b\"]--;\n\
--$counts[\"b\"];\n\
var_dump($counts);",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [\"a\"]=>\n  int(2)\n  [\"b\"]=>\n  int(0)\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_path_read_for_assign_op"));
    assert!(c_source.contains("ptn_runtime_array_path_set_from_assign_op"));
}

#[test]
fn compile_increment_and_decrement_expression_results_to_native_binary() {
    let root = temp_dir("ptn-native-inc-dec-expression-results");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("inc-dec-expression-results.php");
    let output = root.join("inc-dec-expression-results-bin");
    fs::write(
        &input,
        "<?php $value = 1; echo ++$value, ':', $value, \"\\n\"; $old = $value++; echo $old, ':', $value, \"\\n\"; $new = --$value; echo $new, ':', $value, \"\\n\"; echo $value--, ':', $value, \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2:2\n2:3\n2:2\n2:1\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_array_offset_increment_and_decrement_to_native_binary() {
    let root = temp_dir("ptn-native-array-offset-inc-dec");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-offset-inc-dec.php");
    let output = root.join("array-offset-inc-dec-bin");
    fs::write(
        &input,
        "<?php $items = []; $key = 'count'; $items[$key] = 1; echo ++$items[$key], ':', $items[$key], \"\\n\"; echo $items[$key]++, ':', $items[$key], \"\\n\"; $items[$key]--; echo $items[$key], \"\\n\"; $nested = [[\"n\" => 4]]; $nested[0][\"n\"]++; echo $nested[0][\"n\"], \"\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "2:2\n2:3\n2\n5\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_dynamic_inc_dec_expressions_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-inc-dec-expressions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-inc-dec-expressions.php");
    let output = root.join("dynamic-inc-dec-expressions-bin");
    fs::write(
        &input,
        "<?php\n\
$varName = \"counter\";\n\
$counter = 9;\n\
echo ++$$varName, \":\", $counter, \"\\n\";\n\
echo ${$varName}--, \":\", $counter, \"\\n\";\n\
$arrayName = \"items\";\n\
$items = [\"k\" => 3];\n\
echo ${$arrayName}[\"k\"]++, \":\", $items[\"k\"], \"\\n\";\n\
echo --${$arrayName}[\"k\"], \":\", $items[\"k\"], \"\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "10:10\n10:9\n3:4\n3:3\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_scalar_and_string_increment_decrement_to_native_binary() {
    let root = temp_dir("ptn-native-scalar-string-inc-dec");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("scalar-string-inc-dec.php");
    let output = root.join("scalar-string-inc-dec-bin");
    fs::write(
        &input,
        "<?php\n\
$n = null; var_dump($n++, $n);\n\
$n = null; var_dump($n--, $n);\n\
$b = true; var_dump(++$b, $b--, $b);\n\
$s = \"\"; var_dump(++$s);\n\
$s = \"\"; var_dump(--$s);\n\
$s = \"a\"; var_dump($s++, $s);\n\
$s = \"z\"; var_dump(++$s);\n\
$s = \"a9\"; var_dump(++$s);\n\
$s = \"9z\"; var_dump(++$s);\n\
$s = \"099\"; var_dump($s++, $s);\n\
$s = \"1.5\"; var_dump(--$s);\n\
$items = [\"k\" => \"z\"]; var_dump(++$items[\"k\"]);\n\
$varName = \"dyn\"; $dyn = \"y\"; var_dump(++$$varName);\n\
try { $a = []; $a++; } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { $a = []; $a--; } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "NULL\n\
int(1)\n\
NULL\n\
NULL\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
string(1) \"1\"\n\
int(-1)\n\
string(1) \"a\"\n\
string(1) \"b\"\n\
string(2) \"aa\"\n\
string(2) \"b0\"\n\
string(3) \"10a\"\n\
string(3) \"099\"\n\
int(100)\n\
float(0.5)\n\
string(2) \"aa\"\n\
string(1) \"z\"\n\
Cannot increment array\n\
Cannot decrement array\n"
    );
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
fn compile_continue_targeting_switch_warnings_in_uncalled_function_to_native_binary() {
    let root = temp_dir("ptn-native-function-switch-continue-warnings");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("function-switch-continue-warnings.php");
    let output = root.join("function-switch-continue-warnings-bin");
    fs::write(
        &input,
        "<?php

function test() {
    switch ($foo) {
        case 0:
            continue;
        case 1:
            break;
    }

    while ($foo) {
        switch ($bar) {
            case 0:
                continue;
            case 1:
                continue 2;
            case 2:
                break;
        }
    }

    switch ($bar) {
        case 0:
            while ($xyz) {
                continue 2;
            }
        case 1:
            while ($xyz) {
                continue;
            }
        case 2:
            while ($xyz) {
                break 2;
            }
    }

    while ($foo) {
        switch ($bar) {
            case 0:
                while ($xyz) {
                    continue 2;
                }
            case 1:
                while ($xyz) {
                    continue 3;
                }
            case 2:
                while ($xyz) {
                    break 2;
                }
        }
    }
}
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "\nWarning: \"continue\" targeting switch is equivalent to \"break\" in {} on line 6\n\
\nWarning: \"continue\" targeting switch is equivalent to \"break\". Did you mean to use \"continue 2\"? in {} on line 14\n\
\nWarning: \"continue 2\" targeting switch is equivalent to \"break 2\" in {} on line 25\n\
\nWarning: \"continue 2\" targeting switch is equivalent to \"break 2\". Did you mean to use \"continue 3\"? in {} on line 41\n",
            input.display(),
            input.display(),
            input.display(),
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
fn compile_braced_array_interpolation_to_native_binary() {
    let root = temp_dir("ptn-native-braced-array-interpolation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("braced-array-interpolation.php");
    let output = root.join("braced-array-interpolation-bin");
    fs::write(
        &input,
        "<?php\n\
$name = \"Ada\";\n\
$items = [\"name\" => \"compiler\", 0 => \"zero\", \"later\" => \"after\"];\n\
$key = \"later\";\n\
echo \"name={$name} item={$items['name']} first={$items[0]} dynamic={$items[$key]}\\n\";\n\
$empty = [];\n\
echo \"a={$empty['one']}b={$items[0]}c={$empty['two']}d\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "name=Ada item=compiler first=zero dynamic=after\n\
\n\
Warning: Undefined array key \"one\" in ptn on line 7\n\
\n\
Warning: Undefined array key \"two\" in ptn on line 7\n\
a=b=zeroc=d\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_simple_and_legacy_interpolation_to_native_binary() {
    let root = temp_dir("ptn-native-simple-legacy-interpolation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("simple-legacy-interpolation.php");
    let output = root.join("simple-legacy-interpolation-bin");
    fs::write(
        &input,
        "<?php\n\
$items = [\"name\" => \"Ada\", \"later\" => \"compiler\"];\n\
$key = \"later\";\n\
$name = \"legacy\";\n\
echo \"item=$items[$key] bare=$items[name] legacy=${name}!\\n\";",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Using ${var} in strings is deprecated, use {$var} instead in ptn on line 5\n\
item=compiler bare=Ada legacy=legacy!\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_legacy_interpolation_deprecation_precedes_runtime_diagnostic() {
    let root = temp_dir("ptn-native-legacy-interpolation-diagnostic-order");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("legacy-interpolation-diagnostic-order.php");
    let output = root.join("legacy-interpolation-diagnostic-order-bin");
    fs::write(
        &input,
        "<?php\n\
$str = \"world\";\n\
$strS = null;\n\
var_dump(strlen($strS));\n\
var_dump(strlen(\"${str}S\"));\n\
var_dump(strlen(\"{$str}S\"));",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nDeprecated: Using ${var} in strings is deprecated, use {$var} instead in ptn on line 5\n\
\n\
Deprecated: strlen(): Passing null to parameter #1 ($string) of type string is deprecated in ptn on line 4\n\
int(0)\n\
int(6)\n\
int(6)\n"
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}1\n",
            undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
fn compile_direct_null_coalescing_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-null-coalescing-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("null-coalescing-assignment.php");
    let output = root.join("null-coalescing-assignment-bin");
    fs::write(
        &input,
        "<?php\n\
function rhs($label) { echo \"rhs:$label\\n\"; return $label; }\n\
var_dump($missing ??= rhs(\"missing\"));\n\
$nullish = null; var_dump($nullish ??= rhs(\"null\"));\n\
$falsey = false; var_dump($falsey ??= rhs(\"false\"));\n\
$zero = 0; var_dump($zero ??= rhs(\"zero\"));\n\
$existing = \"kept\"; var_dump($existing ??= rhs(\"existing\"));\n\
$standalone = null; $standalone ??= rhs(\"standalone\"); var_dump($standalone);\n\
var_dump($missing, $nullish, $falsey, $zero, $existing);\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "rhs:missing\n\
string(7) \"missing\"\n\
rhs:null\n\
string(4) \"null\"\n\
bool(false)\n\
int(0)\n\
string(4) \"kept\"\n\
rhs:standalone\n\
string(10) \"standalone\"\n\
string(7) \"missing\"\n\
string(4) \"null\"\n\
bool(false)\n\
int(0)\n\
string(4) \"kept\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_offset_null_coalescing_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-offset-null-coalescing-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("offset-null-coalescing-assignment.php");
    let output = root.join("offset-null-coalescing-assignment-bin");
    fs::write(
        &input,
        "<?php\n\
function rhs($label) { echo \"rhs:$label\\n\"; return $label; }\n\
$items = [\"hit\" => \"kept\", \"nullish\" => null, \"nested\" => [\"leaf\" => null]];\n\
var_dump($items[\"hit\"] ??= rhs(\"hit\"));\n\
var_dump($items[\"nullish\"] ??= rhs(\"nullish\"));\n\
var_dump($items[\"missing\"] ??= rhs(\"missing\"));\n\
var_dump($items[\"nested\"][\"leaf\"] ??= rhs(\"nested\"));\n\
$items[\"standalone\"] ??= rhs(\"standalone\");\n\
var_dump($items[\"standalone\"]);\n\
var_dump($undef[\"key\"] ??= rhs(\"undef\"));\n\
$nullbase = null;\n\
var_dump($nullbase[\"key\"] ??= rhs(\"nullbase\"));\n\
$string = \"abc\";\n\
var_dump($string[1] ??= rhs(\"string-hit\"));\n\
var_dump($string[5] ??= rhs(\"string-missing\"));\n\
var_dump($string);\n\
var_dump($items[\"hit\"], $items[\"nullish\"], $items[\"missing\"], $items[\"nested\"][\"leaf\"], $undef[\"key\"], $nullbase[\"key\"]);\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(4) \"kept\"\n\
rhs:nullish\n\
string(7) \"nullish\"\n\
rhs:missing\n\
string(7) \"missing\"\n\
rhs:nested\n\
string(6) \"nested\"\n\
rhs:standalone\n\
string(10) \"standalone\"\n\
rhs:undef\n\
string(5) \"undef\"\n\
rhs:nullbase\n\
string(8) \"nullbase\"\n\
string(1) \"b\"\n\
rhs:string-missing\n\
\n\
Warning: Only the first byte will be assigned to the string offset in ptn on line 15\n\
string(1) \"s\"\n\
string(6) \"abc  s\"\n\
string(4) \"kept\"\n\
string(7) \"nullish\"\n\
string(7) \"missing\"\n\
string(6) \"nested\"\n\
string(5) \"undef\"\n\
string(8) \"nullbase\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_path_lookup_quiet(&runtime"));
    assert!(c_source.contains("ptn_runtime_array_path_set(&runtime"));
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}0\n{}[]\n",
            undefined_variable_warnings(&input, &[("total", 1), ("missing_number", 1)]),
            undefined_variable_warnings(&input, &[("text", 1), ("missing_text", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
    assert!(main_body.contains("ptn_value_drop(&ptn_tmp_"));
}

#[test]
fn compile_cow_shared_payload_mutations_to_native_binary() {
    let root = temp_dir("ptn-native-cow-shared-payload-mutations");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("cow-shared-payload-mutations.php");
    let output = root.join("cow-shared-payload-mutations-bin");
    fs::write(
        &input,
        "<?php
$a = [1, 2];
$b = $a;
$b[] = 3;
var_dump($a, $b);

$c = [10, 20];
$d = $c;
var_dump(array_shift($d));
var_dump($c, $d);

$s = \"ab\";
$t = $s;
$t[0] = \"Z\";
var_dump($s, $t);

function passthrough($value) {
    $local = $value;
    $local[] = \"fn\";
    return $local;
}
$source = [\"x\"];
$result = passthrough($source);
var_dump($source, $result);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\narray(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n}\nint(10)\narray(2) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n}\narray(1) {\n  [0]=>\n  int(20)\n}\nstring(2) \"ab\"\nstring(2) \"Zb\"\narray(1) {\n  [0]=>\n  string(1) \"x\"\n}\narray(2) {\n  [0]=>\n  string(1) \"x\"\n  [1]=>\n  string(2) \"fn\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    let main_start = c_source
        .find("\nint main(void)")
        .expect("generated C should contain main");
    let main_body = &c_source[main_start..];
    assert!(c_source.contains("static PTN_UNUSED PtnValue ptn_value_share(PtnValue value)"));
    assert!(c_source.contains("static PTN_UNUSED void ptn_value_drop(PtnValue *value)"));
    assert!(
        c_source.contains("static PTN_UNUSED PtnArray *ptn_value_detach_array(PtnValue *value)")
    );
    assert!(main_body.contains("ptn_runtime_array_shift_variable(&runtime, \"d\""));
    assert!(main_body.contains("ptn_value_share(ptn_tmp_"));
    assert!(main_body.contains("ptn_value_drop(&ptn_tmp_"));
}

#[test]
fn compile_function_boundary_cow_payloads_to_native_binary() {
    let root = temp_dir("ptn-native-function-boundary-cow-payloads");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("function-boundary-cow-payloads.php");
    let output = root.join("function-boundary-cow-payloads-bin");
    fs::write(
        &input,
        "<?php
function mutate_by_value($arr, $str) {
    $local_arr = $arr;
    $local_str = $str;
    $local_extra = func_get_arg(2);
    $arr[\"param\"] = \"changed\";
    $local_arr[] = \"local\";
    $str[0] = \"Z\";
    $local_str[1] = \"Y\";
    $local_extra[] = \"extra-local\";
    return [$arr, $local_arr, $str, $local_str, func_get_arg(0), func_get_arg(1), func_get_arg(2), $local_extra];
}

function identity($value) { return $value; }

function recurse_array($value, $depth) {
    if ($depth <= 0) {
        return $value;
    }
    $again = recurse_array($value, $depth - 1);
    $again[] = \"depth-\" . $depth;
    return $again;
}

$base = [\"k\" => \"v\"];
$text = \"abc\";
$extra = [\"e\" => \"v\"];
$result = mutate_by_value($base, $text, $extra);
var_dump($base, $text, $extra, $result);

$returned_arr = identity($base);
$returned_arr[\"r\"] = \"ret\";
$returned_str = identity($text);
$returned_str[2] = \"Z\";
var_dump($base, $returned_arr, $text, $returned_str);

$recursive_seed = [\"seed\"];
$recursive = recurse_array($recursive_seed, 2);
var_dump($recursive_seed, $recursive);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(1) {\n",
            "  [\"k\"]=>\n",
            "  string(1) \"v\"\n",
            "}\n",
            "string(3) \"abc\"\n",
            "array(1) {\n",
            "  [\"e\"]=>\n",
            "  string(1) \"v\"\n",
            "}\n",
            "array(8) {\n",
            "  [0]=>\n",
            "  array(2) {\n",
            "    [\"k\"]=>\n",
            "    string(1) \"v\"\n",
            "    [\"param\"]=>\n",
            "    string(7) \"changed\"\n",
            "  }\n",
            "  [1]=>\n",
            "  array(2) {\n",
            "    [\"k\"]=>\n",
            "    string(1) \"v\"\n",
            "    [0]=>\n",
            "    string(5) \"local\"\n",
            "  }\n",
            "  [2]=>\n",
            "  string(3) \"Zbc\"\n",
            "  [3]=>\n",
            "  string(3) \"aYc\"\n",
            "  [4]=>\n",
            "  array(2) {\n",
            "    [\"k\"]=>\n",
            "    string(1) \"v\"\n",
            "    [\"param\"]=>\n",
            "    string(7) \"changed\"\n",
            "  }\n",
            "  [5]=>\n",
            "  string(3) \"Zbc\"\n",
            "  [6]=>\n",
            "  array(1) {\n",
            "    [\"e\"]=>\n",
            "    string(1) \"v\"\n",
            "  }\n",
            "  [7]=>\n",
            "  array(2) {\n",
            "    [\"e\"]=>\n",
            "    string(1) \"v\"\n",
            "    [0]=>\n",
            "    string(11) \"extra-local\"\n",
            "  }\n",
            "}\n",
            "array(1) {\n",
            "  [\"k\"]=>\n",
            "  string(1) \"v\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"k\"]=>\n",
            "  string(1) \"v\"\n",
            "  [\"r\"]=>\n",
            "  string(3) \"ret\"\n",
            "}\n",
            "string(3) \"abc\"\n",
            "string(3) \"abZ\"\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(4) \"seed\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  string(4) \"seed\"\n",
            "  [1]=>\n",
            "  string(7) \"depth-1\"\n",
            "  [2]=>\n",
            "  string(7) \"depth-2\"\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_set_call_frame"));
    assert!(c_source.contains("ptn_return_value = ptn_value_clone(ptn_value_deref("));
    assert!(c_source.contains("ptn_runtime_write_variable(&runtime, \"arr\", args[0]);"));
    assert!(c_source.contains("ptn_runtime_write_variable(&runtime, \"str\", args[1]);"));
    assert!(
        c_source.contains("static PTN_UNUSED PtnArray *ptn_value_detach_array(PtnValue *value)")
    );
}

#[test]
fn compile_array_reduce_callback_by_ref_return_to_native_binary() {
    let root = temp_dir("ptn-native-array-reduce-by-ref-callback");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reduce-by-ref-callback.php");
    let output = root.join("array-reduce-by-ref-callback-bin");
    fs::write(
        &input,
        "<?php
function &pick_reduce_value($carry, $value) {
    return $value;
}

$array = [1, 2];
var_dump(array_reduce($array, \"pick_reduce_value\", 0));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "int(2)\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reduce"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, 2, callback_args"));
    assert!(c_source.contains("carry = ptn_value_clone_deref(callback_result);"));
}

#[test]
fn compile_array_reduce_variadic_callback_to_native_binary() {
    let root = temp_dir("ptn-native-array-reduce-variadic-callback");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reduce-variadic-callback.php");
    let output = root.join("array-reduce-variadic-callback-bin");
    fs::write(
        &input,
        "<?php
function variadic_sum($carry, ...$values) {
    var_dump($values, func_get_arg(1));
    return $carry + $values[0];
}

var_dump(array_reduce([1, 2], \"variadic_sum\", 0));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "array(1) {\n  [0]=>\n  int(1)\n}\nint(1)\narray(1) {\n  [0]=>\n  int(2)\n}\nint(2)\nint(3)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reduce"));
    assert!(c_source.contains("ptn_array_set_entry(ptn_variadic_1.as.array"));
}

#[test]
fn compile_array_map_string_callable_to_native_binary() {
    let root = temp_dir("ptn-native-array-map-string-callable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-map-string-callable.php");
    let output = root.join("array-map-string-callable-bin");
    fs::write(
        &input,
        "<?php
function decorate($value) {
    return \"v=\" . $value;
}

function pair_values($left, $right) {
    return [$left, $right];
}

$callback = \"decorate\";
var_dump(array_map($callback, [\"a\" => 1, \"b\" => 2]));
var_dump(array_map(\"pair_values\", [1, 2, 3], [10]));
var_dump(array_map(null, [\"x\" => \"left\", \"y\" => \"right\"]));
var_dump(array_map(null, [\"x\" => 1, \"y\" => 2], [10]));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(2) {\n",
            "  [\"a\"]=>\n",
            "  string(3) \"v=1\"\n",
            "  [\"b\"]=>\n",
            "  string(3) \"v=2\"\n",
            "}\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "    [1]=>\n",
            "    int(10)\n",
            "  }\n",
            "  [1]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(2)\n",
            "    [1]=>\n",
            "    NULL\n",
            "  }\n",
            "  [2]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(3)\n",
            "    [1]=>\n",
            "    NULL\n",
            "  }\n",
            "}\n",
            "array(2) {\n",
            "  [\"x\"]=>\n",
            "  string(4) \"left\"\n",
            "  [\"y\"]=>\n",
            "  string(5) \"right\"\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "    [1]=>\n",
            "    int(10)\n",
            "  }\n",
            "  [1]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(2)\n",
            "    [1]=>\n",
            "    NULL\n",
            "  }\n",
            "}\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_map"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, array_count"));
    assert!(c_source.contains("ptn_array_map_result_key"));
}

#[test]
fn compile_namespaced_registered_internals_fall_back_to_global_runtime() {
    let root = temp_dir("ptn-native-namespaced-registered-internal-fallback");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("namespaced-registered-internal-fallback.php");
    let output = root.join("namespaced-registered-internal-fallback-bin");
    fs::write(
        &input,
        "<?php
namespace Demo;

class Worker {
    public function run() {
        return \"ok\";
    }
}

function bump($value) {
    return $value + 1;
}

function add($carry, $value) {
    return $carry + $value;
}

function label($value) {
    return $value . \"!\";
}

var_dump(array_map(\"Demo\\\\bump\", [1, 2]));
var_dump(array_reduce([1, 2, 3], \"Demo\\\\add\", 0));
var_dump(call_user_func(\"Demo\\\\label\", \"x\"));
var_dump(is_callable(\"strlen\"));
var_dump(class_exists(\"Demo\\\\Worker\"));
var_dump(method_exists(\"Demo\\\\Worker\", \"run\"));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "array(2) {\n",
            "  [0]=>\n",
            "  int(2)\n",
            "  [1]=>\n",
            "  int(3)\n",
            "}\n",
            "int(6)\n",
            "string(2) \"x!\"\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_call_function(&runtime, \"array_map\""));
    assert!(c_source.contains("ptn_call_function(&runtime, \"array_reduce\""));
    assert!(c_source.contains("ptn_call_function(&runtime, \"call_user_func\""));
    assert!(c_source.contains("ptn_call_function(&runtime, \"class_exists\""));
}

#[test]
fn compile_call_user_func_string_callable_to_native_binary() {
    let root = temp_dir("ptn-native-call-user-func-string-callable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("call-user-func-string-callable.php");
    let output = root.join("call-user-func-string-callable-bin");
    fs::write(
        &input,
        "<?php
function join_pair($left, $right) {
    echo $left, \":\", $right, \"\\n\";
    return $left . \"-\" . $right;
}

function wrapper($callable, $value) {
    return call_user_func($callable, $value, \"tail\");
}

function inspect_args($first) {
    return func_num_args() . \":\" . func_get_arg(1);
}

var_dump(wrapper(\"join_pair\", \"head\"));
var_dump(call_user_func(\"strlen\", \"abcd\"));
var_dump(call_user_func(\"inspect_args\", \"one\", \"two\"));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "head:tail\nstring(9) \"head-tail\"\nint(4)\nstring(5) \"2:two\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_call_user_func"));
    assert!(c_source.contains("ptn_call_callable("));
    assert!(c_source.contains("args[0]"));
    assert!(c_source.contains("argc - 1"));
}

#[test]
fn compile_array_walk_callback_global_swap_to_native_binary() {
    let root = temp_dir("ptn-native-array-walk-global-swap");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-walk-global-swap.php");
    let output = root.join("array-walk-global-swap-bin");
    fs::write(
        &input,
        "<?php
function walk_swap(&$value, $key) {
    var_dump($value);
    if ($value == 2) {
        $GLOBALS[\"array\"] = $GLOBALS[\"array2\"];
    }
    $value *= 10;
}

$array = [1, 2, 3];
$array2 = [4, 5];
array_walk($array, \"walk_swap\");
var_dump($array, $array2);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "int(1)\n",
            "int(2)\n",
            "int(4)\n",
            "int(5)\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(40)\n",
            "  [1]=>\n",
            "  int(50)\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(4)\n",
            "  [1]=>\n",
            "  int(5)\n",
            "}\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_walk_variable(&runtime, \"array\""));
    assert!(c_source.contains("ptn_runtime_globals_array_path_set_impl"));
}

#[test]
fn compile_array_reduce_static_method_callables_to_native_binary() {
    let root = temp_dir("ptn-native-array-reduce-static-method-callables");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reduce-static-method-callables.php");
    let output = root.join("array-reduce-static-method-callables-bin");
    fs::write(
        &input,
        "<?php
class Reducer {
    public static function combine($carry, $value) {
        return $carry + $value;
    }
}

echo array_reduce([1, 2, 3], [\"Reducer\", \"combine\"], 0), \"\\n\";
echo array_reduce([4, 5], \"Reducer::combine\", 1), \"\\n\";
echo array_reduce([6], [1 => \"combine\", 0 => \"Reducer\"], 0), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "6\n10\n6\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reduce"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, 2, callback_args"));
    assert!(c_source.contains("Reducer::combine"));
}

#[test]
fn compile_array_reduce_instance_method_callable_to_native_binary() {
    let root = temp_dir("ptn-native-array-reduce-instance-method-callable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reduce-instance-method-callable.php");
    let output = root.join("array-reduce-instance-method-callable-bin");
    fs::write(
        &input,
        "<?php
class Reducer {
    public function combine($carry, $value) {
        return $carry . \":\" . $value;
    }
}

$reducer = new Reducer();
echo array_reduce([\"a\", \"b\"], [$reducer, \"combine\"], \"seed\"), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "seed:a:b\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reduce"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, 2, callback_args"));
    assert!(c_source.contains("receiver.type == PTN_OBJECT || receiver.type == PTN_EXCEPTION"));
    assert!(c_source.contains("ptn_call_declared_method(runtime, receiver"));
}

#[test]
fn compile_anonymous_function_dynamic_call_to_native_binary() {
    let root = temp_dir("ptn-native-anonymous-function-dynamic-call");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("anonymous-function-dynamic-call.php");
    let output = root.join("anonymous-function-dynamic-call-bin");
    fs::write(
        &input,
        "<?php
$callback = function ($value) {
    return $value + 2;
};
echo $callback(3), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "5\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_closure("));
    assert!(c_source.contains("static PTN_UNUSED PtnValue ptn_call_callable("));
    assert!(c_source.contains("resolved.type == PTN_CLOSURE"));
}

#[test]
fn compile_nested_anonymous_function_dynamic_call_to_native_binary() {
    let root = temp_dir("ptn-native-nested-anonymous-function-dynamic-call");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("nested-anonymous-function-dynamic-call.php");
    let output = root.join("nested-anonymous-function-dynamic-call-bin");
    fs::write(
        &input,
        "<?php
$outer = function () {
    $inner = function () {
        return 7;
    };
    return $inner() + 3;
};
echo $outer(), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "10\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_closure("));
}

#[test]
fn compile_array_reduce_anonymous_callback_by_ref_return_to_native_binary() {
    let root = temp_dir("ptn-native-array-reduce-anonymous-by-ref-callback");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-reduce-anonymous-by-ref-callback.php");
    let output = root.join("array-reduce-anonymous-by-ref-callback-bin");
    fs::write(
        &input,
        "<?php
$array = [1, 2];
var_dump(array_reduce($array, function &($carry, $value) {
    return $value;
}, 0));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "int(2)\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_array_reduce"));
    assert!(c_source.contains("ptn_call_callable(runtime, callback, 2, callback_args"));
    assert!(c_source.contains("ptn_closure("));
}

#[test]
fn compile_anonymous_function_use_value_capture_to_native_binary() {
    let root = temp_dir("ptn-native-anonymous-function-use-value-capture");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("anonymous-function-use-value-capture.php");
    let output = root.join("anonymous-function-use-value-capture-bin");
    fs::write(
        &input,
        "<?php
$x = 2;
$callback = function ($value) use ($x) {
    $x++;
    return $value + $x;
};
$x = 10;
echo $callback(3), \"\\n\";
echo $callback(4), \"\\n\";
echo $x, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "6\n7\n10\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_closure_set_capture("));
    assert!(c_source.contains("ptn_runtime_import_closure_captures(&runtime, receiver);"));
}

#[test]
fn compile_anonymous_function_use_reference_capture_to_native_binary() {
    let root = temp_dir("ptn-native-anonymous-function-use-reference-capture");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("anonymous-function-use-reference-capture.php");
    let output = root.join("anonymous-function-use-reference-capture-bin");
    fs::write(
        &input,
        "<?php
$x = 1;
$callback = function () use (&$x) {
    $x++;
    return $x;
};
echo $callback(), \"\\n\";
echo $callback(), \"\\n\";
echo $x, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "2\n3\n3\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_closure_bind_capture_reference("));
    assert!(c_source.contains("ptn_runtime_reference_for_variable(&runtime, \"x\")"));
}

#[test]
fn compile_array_walk_closure_use_capture_global_swap_to_native_binary() {
    let root = temp_dir("ptn-native-array-walk-closure-use-global-swap");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("array-walk-closure-use-global-swap.php");
    let output = root.join("array-walk-closure-use-global-swap-bin");
    fs::write(
        &input,
        "<?php
$array = [1, 2, 3];
$array2 = [4, 5];
array_walk($array, function (&$value, $key) use ($array2) {
    var_dump($value);
    if ($value == 2) { $GLOBALS[\"array\"] = $array2; }
    $value *= 10;
});
var_dump($array, $array2);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(2)\nint(4)\nint(5)\narray(2) {\n  [0]=>\n  int(40)\n  [1]=>\n  int(50)\n}\narray(2) {\n  [0]=>\n  int(4)\n  [1]=>\n  int(5)\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_array_walk_variable(&runtime, \"array\""));
    assert!(c_source.contains("ptn_call_callable("));
    assert!(c_source.contains("ptn_closure_set_capture("));
}

#[test]
fn parser_accepts_stdclass_property_reads_and_writes() {
    let program = parser::parse(
        "<?php\n\
$object = new stdClass;\n\
$object->value = 7;\n\
echo $object->value;\n\
$object->missing ??= 8;\n",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 4);

    let Statement::Expression { expression, .. } = &program.statements[1] else {
        panic!("expected property assignment expression statement");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::Property { name, .. },
            ..
        } if name == "value"
    ));

    let Statement::Echo { expressions, .. } = &program.statements[2] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::PropertyFetch { name, .. } if name == "value"
    ));

    let Statement::Expression { expression, .. } = &program.statements[3] else {
        panic!("expected property null coalescing assignment expression statement");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::Property { name, .. },
            op: AssignmentOp::CoalesceAssign,
            ..
        } if name == "missing"
    ));
}

#[test]
fn compile_stdclass_property_reads_and_writes_to_native_binary() {
    let root = temp_dir("ptn-native-stdclass-property-access");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("stdclass-property-access.php");
    let output = root.join("stdclass-property-access-bin");
    fs::write(
        &input,
        "<?php
$object = new stdClass;
$object->value = 7;
$alias = $object;
$alias->value = $object->value + 5;
var_dump($object->value);
var_dump($alias->value = 21);
var_dump($object->value);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(12)\nint(21)\nint(21)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_new_object(&runtime, \"stdClass\""));
    assert!(c_source.contains("ptn_object_write_property(&runtime"));
    assert!(c_source.contains("ptn_object_read_property(&runtime"));
}

#[test]
fn compile_stdclass_property_null_coalescing_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-stdclass-property-coalesce-assign");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("stdclass-property-coalesce-assign.php");
    let output = root.join("stdclass-property-coalesce-assign-bin");
    fs::write(
        &input,
        "<?php
function receiver($object, $label) {
    echo \"receiver:$label\\n\";
    return $object;
}
function rhs($label) {
    echo \"rhs:$label\\n\";
    return $label;
}

$object = new stdClass;
var_dump(receiver($object, \"missing\")->missing ??= rhs(\"missing\"));
$object->nullish = null;
var_dump(receiver($object, \"nullish\")->nullish ??= rhs(\"nullish\"));
$object->hit = \"kept\";
var_dump(receiver($object, \"hit\")->hit ??= rhs(\"hit\"));
var_dump($object->missing, $object->nullish, $object->hit);

$scalar = 3;
try {
    var_dump($scalar->bad ??= rhs(\"bad\"));
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "receiver:missing\n\
rhs:missing\n\
receiver:missing\n\
string(7) \"missing\"\n\
receiver:nullish\n\
rhs:nullish\n\
receiver:nullish\n\
string(7) \"nullish\"\n\
receiver:hit\n\
string(4) \"kept\"\n\
string(7) \"missing\"\n\
string(7) \"nullish\"\n\
string(4) \"kept\"\n\
rhs:bad\n\
Attempt to assign property \"bad\" on int\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_property_lookup_quiet(&runtime"));
    assert!(c_source.contains("ptn_object_write_property(&runtime"));
}

#[test]
fn compile_property_isset_empty_and_coalesce_to_native_binary() {
    let root = temp_dir("ptn-native-property-isset-empty-coalesce");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("property-isset-empty-coalesce.php");
    let output = root.join("property-isset-empty-coalesce-bin");
    fs::write(
        &input,
        "<?php
class C {
    private $secret = 1;

    public function probe() {
        var_dump(isset($this->secret), empty($this->secret), $this->secret ?? \"fallback\");
    }
}

function receiver($object, $label) {
    echo \"receiver:$label\\n\";
    return $object;
}

$object = new stdClass();
$object->x = null;
$object->zero = \"0\";
$object->truthy = 5;
var_dump(isset(receiver($object, \"truthy\")->truthy));
var_dump(isset(receiver($object, \"nullish\")->x));
var_dump(isset(receiver($object, \"missing\")->missing));
var_dump(empty(receiver($object, \"zero\")->zero));
var_dump(empty(receiver($object, \"truthy-empty\")->truthy));
var_dump(empty(receiver($object, \"missing-empty\")->missing));
var_dump(receiver($object, \"coalesce-hit\")->truthy ?? \"fallback\");
var_dump(receiver($object, \"coalesce-null\")->x ?? \"fallback\");
var_dump(receiver($object, \"coalesce-missing\")->missing ?? \"fallback\");

$scalar = 3;
var_dump(isset($scalar->prop), empty($scalar->prop), $scalar->prop ?? \"fallback\");

$declared = new C();
$declared->probe();
var_dump(isset($declared->secret), empty($declared->secret), $declared->secret ?? \"fallback\");
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "receiver:truthy\n",
            "bool(true)\n",
            "receiver:nullish\n",
            "bool(false)\n",
            "receiver:missing\n",
            "bool(false)\n",
            "receiver:zero\n",
            "bool(true)\n",
            "receiver:truthy-empty\n",
            "bool(false)\n",
            "receiver:missing-empty\n",
            "bool(true)\n",
            "receiver:coalesce-hit\n",
            "int(5)\n",
            "receiver:coalesce-null\n",
            "string(8) \"fallback\"\n",
            "receiver:coalesce-missing\n",
            "string(8) \"fallback\"\n",
            "bool(false)\n",
            "bool(true)\n",
            "string(8) \"fallback\"\n",
            "bool(true)\n",
            "bool(false)\n",
            "int(1)\n",
            "bool(false)\n",
            "bool(true)\n",
            "string(8) \"fallback\"\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_property_probe_quiet(&runtime"));
}

#[test]
fn compile_class_constant_reads_to_native_binary() {
    let root = temp_dir("ptn-native-class-constant-reads");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("class-constant-reads.php");
    let output = root.join("class-constant-reads-bin");
    fs::write(
        &input,
        "<?php
class Sample {
    const X = 42, Data = [\"answer\" => 42];
    public const Label = \"hello\";

    public static function label() {
        return self::Label;
    }
}

echo Sample::X, \"\\n\";
var_dump(Sample::Data);
echo Sample::label(), \"\\n\";
var_dump(defined(\"sample::Label\"), constant(\"sample::Label\"));
try {
    var_dump(Sample::label);
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "42\n",
            "array(1) {\n",
            "  [\"answer\"]=>\n",
            "  int(42)\n",
            "}\n",
            "hello\n",
            "bool(true)\n",
            "string(5) \"hello\"\n",
            "Undefined constant Sample::label\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_define_class_constant(&runtime"));
    assert!(c_source.contains("ptn_runtime_read_class_constant(&runtime"));
}

#[test]
fn compile_static_property_reads_and_writes_to_native_binary() {
    let root = temp_dir("ptn-native-static-property-access");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("static-property-access.php");
    let output = root.join("static-property-access-bin");
    fs::write(
        &input,
        "<?php
class Counter {
    public static $value = 1;

    public static function bump() {
        self::$value = self::$value + 1;
        return self::$value;
    }
}

echo Counter::$value, \"\\n\";
Counter::$value = 5;
echo Counter::bump(), \"\\n\";
echo Counter::$value, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "1\n6\n6\n");
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_define_static_property(&runtime"));
    assert!(c_source.contains("ptn_runtime_read_static_property(&runtime"));
    assert!(c_source.contains("ptn_runtime_write_static_property(&runtime"));
}

#[test]
fn compile_property_exists_metadata_to_native_binary() {
    let root = temp_dir("ptn-native-property-exists-metadata");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("property-exists-metadata.php");
    let output = root.join("property-exists-metadata-bin");
    fs::write(
        &input,
        "<?php
class Base {
    private $basePrivate = 1;
    protected $shared = 2;
    public static $baseStatic = 3;
    private static $hiddenStatic = 4;

    public static function inheritedProbe() {
        return property_exists(\"Child\", \"baseStatic\");
    }
}

class Child extends Base {
    private $childPrivate = 5;
    public $visible = 6;
    public static $childStatic = 7;
}

$child = new Child();
$child->dynamic = 8;
$std = new stdClass();
$std->value = null;

var_dump(function_exists(\"property_exists\"));
var_dump(Child::inheritedProbe());
var_dump(property_exists(\"Base\", \"basePrivate\"));
var_dump(property_exists(\"Child\", \"basePrivate\"));
var_dump(property_exists(\"Child\", \"shared\"));
var_dump(property_exists(\"Child\", \"baseStatic\"));
var_dump(property_exists(\"Child\", \"hiddenStatic\"));
var_dump(property_exists(\"Child\", \"childPrivate\"));
var_dump(property_exists(\"stdClass\", \"value\"));
var_dump(property_exists($child, \"childPrivate\"));
var_dump(property_exists($child, \"basePrivate\"));
var_dump(property_exists($child, \"shared\"));
var_dump(property_exists($child, \"baseStatic\"));
var_dump(property_exists($child, \"dynamic\"));
var_dump(property_exists($std, \"value\"));
var_dump(property_exists($std, \"missing\"));

try {
    var_dump(property_exists(1, \"value\"));
} catch (TypeError $e) {
    echo $e->getMessage(), \"\\n\";
}

try {
    var_dump(property_exists(\"Child\", new stdClass()));
} catch (TypeError $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "property_exists(): Argument #1 ($object_or_class) must be of type object|string, int given\n",
            "property_exists(): Argument #2 ($property) must be of type string, stdClass given\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_declared_class_property_exists"));
    assert!(c_source.contains("ptn_object_public_property_slot_exists"));
}

#[test]
fn compile_property_and_static_property_inc_dec_to_native_binary() {
    let root = temp_dir("ptn-native-property-static-inc-dec");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("property-static-inc-dec.php");
    let output = root.join("property-static-inc-dec-bin");
    fs::write(
        &input,
        "<?php
class Box {
    public $value = 1;
    private $secret = 4;
    public static $count = 1;

    public function bump() {
        var_dump($this->secret++);
        var_dump(++$this->secret);
    }

    public static function bumpStatic() {
        var_dump(self::$count++);
        var_dump(++self::$count);
    }
}

$box = new Box();
var_dump($box->value++);
var_dump($box->value);
var_dump(++$box->value);
$box->value--;
var_dump($box->value);
$box->bump();
var_dump(Box::$count++);
var_dump(++Box::$count);
Box::$count--;
var_dump(Box::$count);
Box::bumpStatic();
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(1)\nint(2)\nint(3)\nint(2)\nint(4)\nint(6)\nint(1)\nint(3)\nint(2)\nint(2)\nint(4)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_read_property(&runtime"));
    assert!(c_source.contains("ptn_object_write_property(&runtime"));
    assert!(c_source.contains("ptn_runtime_read_static_property(&runtime"));
    assert!(c_source.contains("ptn_runtime_write_static_property(&runtime"));
    assert!(c_source.contains("ptn_increment_value(&runtime"));
    assert!(c_source.contains("ptn_decrement_value(&runtime"));
}

#[test]
fn compile_static_property_undeclared_diagnostics_to_native_binary() {
    let root = temp_dir("ptn-native-static-property-diagnostics");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("static-property-diagnostics.php");
    let output = root.join("static-property-diagnostics-bin");
    fs::write(
        &input,
        "<?php
class Known {
    public static $value;
}

try {
    echo Known::$missing;
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}

try {
    Missing::$value = 1;
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Access to undeclared static property Known::$missing\nAccess to undeclared static property Missing::$value\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_static_property_isset_empty_and_coalesce_to_native_binary() {
    let root = temp_dir("ptn-native-static-property-isset-empty-coalesce");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("static-property-isset-empty-coalesce.php");
    let output = root.join("static-property-isset-empty-coalesce-bin");
    fs::write(
        &input,
        "<?php
class Counter {
    public static $nullish = null;
    public static $zero = \"0\";
    public static $truthy = 5;

    public static function probe() {
        var_dump(isset(self::$truthy), empty(self::$zero), self::$missing ?? \"fallback\");
    }
}

var_dump(isset(Counter::$truthy), isset(Counter::$nullish), isset(Counter::$missing));
var_dump(empty(Counter::$zero), empty(Counter::$truthy), empty(Counter::$missing));
var_dump(Counter::$truthy ?? \"fallback\", Counter::$nullish ?? \"fallback\", Counter::$missing ?? \"fallback\");
Counter::probe();
try {
    echo Counter::$missing;
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "int(5)\n",
            "string(8) \"fallback\"\n",
            "string(8) \"fallback\"\n",
            "bool(true)\n",
            "bool(true)\n",
            "string(8) \"fallback\"\n",
            "Access to undeclared static property Counter::$missing\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_read_static_property_quiet(&runtime"));
    assert!(c_source.contains("ptn_runtime_read_static_property(&runtime"));
}

#[test]
fn compile_static_property_null_coalescing_assignment_to_native_binary() {
    let root = temp_dir("ptn-native-static-property-coalesce-assign");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("static-property-coalesce-assign.php");
    let output = root.join("static-property-coalesce-assign-bin");
    fs::write(
        &input,
        "<?php
class Counter {
    public static $nullish = null;
    public static $hit = \"kept\";
    public static $selfValue;

    public static function initSelf() {
        var_dump(self::$selfValue ??= rhs(\"self\"));
        var_dump(self::$selfValue ??= rhs(\"self-hit\"));
    }
}

function rhs($label) {
    echo \"rhs:$label\\n\";
    return $label;
}

var_dump(Counter::$nullish ??= rhs(\"nullish\"));
var_dump(Counter::$hit ??= rhs(\"hit\"));
var_dump(Counter::$nullish, Counter::$hit);
Counter::initSelf();
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "rhs:nullish\n",
            "string(7) \"nullish\"\n",
            "string(4) \"kept\"\n",
            "string(7) \"nullish\"\n",
            "string(4) \"kept\"\n",
            "rhs:self\n",
            "string(4) \"self\"\n",
            "string(4) \"self\"\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_read_static_property_quiet(&runtime"));
    assert!(c_source.contains("ptn_runtime_write_static_property(&runtime"));
}

#[test]
fn compile_callback_shaped_object_property_read_to_native_binary() {
    let root = temp_dir("ptn-native-callback-object-property-read");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("callback-object-property-read.php");
    let output = root.join("callback-object-property-read-bin");
    fs::write(
        &input,
        "<?php
function read_member($object) {
    return $object->value;
}

$object = new stdClass;
$object->value = \"callback\";
var_dump(call_user_func(\"read_member\", $object));
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(8) \"callback\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_internal_call_user_func"));
    assert!(c_source.contains("ptn_object_read_property(&runtime"));
}

#[test]
fn compile_typed_by_ref_return_separates_function_boundaries_to_native_binary() {
    let root = temp_dir("ptn-native-typed-by-ref-return-separation");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("typed-by-ref-return-separation.php");
    let output = root.join("typed-by-ref-return-separation-bin");
    fs::write(
        &input,
        "<?php
function test1(&$abc) : string {
    return $abc;
}

function &test2(int $abc) : string {
    return $abc;
}

function &test3(int &$abc) : string {
    return $abc;
}

function test4(string $abc) : string {
    return $abc;
}

$a = 123;

var_dump(test4(456));
var_dump(test1($a));
var_dump($a);
var_dump(test2($a));
var_dump($a);
var_dump(test3($a));
var_dump($a);
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "string(3) \"456\"\n",
            "string(3) \"123\"\n",
            "int(123)\n",
            "string(3) \"123\"\n",
            "int(123)\n",
            "string(3) \"123\"\n",
            "string(3) \"123\"\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(
        c_source.contains("PtnValue ptn_typed_return_value = ptn_cast_string(ptn_return_value);")
    );
    assert!(c_source
        .contains("ptn_reference_assign(ptn_return_value.as.reference, ptn_typed_return_value);"));
}

#[test]
fn compile_by_ref_return_boundary_cases_to_native_binary() {
    let root = temp_dir("ptn-native-by-ref-return-boundaries");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("by-ref-return-boundaries.php");
    let output = root.join("by-ref-return-boundaries-bin");
    fs::write(
        &input,
        "<?php
function &id(&$value) {
    return $value;
}

function &slot(&$items) {
    return $items[\"k\"];
}

function &local_box() {
    $local = 41;
    return $local;
}

function &as_string(&$value): string {
    return $value;
}

function wrap_copy(&$value) {
    return id($value);
}

$value = 1;
$alias =& id($value);
$alias = 2;
echo $value, \"|\", $alias, \"\\n\";

$copy = id($value);
$copy = 3;
echo $value, \"|\", $copy, \"\\n\";

$items = [\"k\" => 4];
$slot =& slot($items);
$slot = 5;
echo $items[\"k\"], \"|\", $slot, \"\\n\";

$local =& local_box();
$local = 42;
echo $local, \"\\n\";

$typed = 123;
$typed_alias =& as_string($typed);
echo gettype($typed), \":\", $typed, \"|\", gettype($typed_alias), \":\", $typed_alias, \"\\n\";
$typed_alias = \"abc\";
echo gettype($typed), \":\", $typed, \"\\n\";

$wrapped = 7;
$wrapped_copy = wrap_copy($wrapped);
$wrapped_copy = 8;
echo $wrapped, \"|\", $wrapped_copy, \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "2|2\n",
            "2|3\n",
            "5|5\n",
            "42\n",
            "string:123|string:123\n",
            "string:abc\n",
            "7|8\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_reference_for_variable(&runtime, \"value\")"));
    assert!(c_source.contains("ptn_runtime_reference_for_array_path(&runtime, \"items\""));
    assert!(c_source.contains("ptn_reference_source_or_value"));
    assert!(c_source.contains("ptn_value_clone(ptn_value_deref("));
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
fn compile_arithmetic_rejects_non_numeric_operands_to_native_binary() {
    let root = temp_dir("ptn-native-arithmetic-non-numeric-operands");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("arithmetic-non-numeric-operands.php");
    let output = root.join("arithmetic-non-numeric-operands-bin");
    fs::write(
        &input,
        "<?php\n\
try { var_dump(\"abc\" + 1); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(1 + \"abc\"); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump([1] + 2); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(\"abc\" * 2); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump(\"123abc\" + \"abc\"); } catch (\\TypeError $e) { echo $e->getMessage(), \"\\n\"; }\n\
var_dump(\"123abc\" + 2);\n\
var_dump(\"3.5x\" * 2);\n\
$object = new stdClass;\n\
try { var_dump($object + [1]); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
try { var_dump([1] + $object); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Unsupported operand types: string + int\n\
Unsupported operand types: int + string\n\
Unsupported operand types: array + int\n\
Unsupported operand types: string * int\n\
\n\
Warning: A non-numeric value encountered in ptn on line 6\n\
Unsupported operand types: string + string\n\
\n\
Warning: A non-numeric value encountered in ptn on line 7\n\
int(125)\n\
\n\
Warning: A non-numeric value encountered in ptn on line 8\n\
float(7)\n\
Unsupported operand types: stdClass + array\n\
Unsupported operand types: array + stdClass\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_add(&runtime, "));
    assert!(c_source.contains("ptn_multiply(&runtime, "));
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
        "int(11)\nfloat(25)\nint(5)\nint(4)\nint(2)\nbool(true)\nint(1)\nfloat(10)\nfloat(2.5)\n"
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}0\n",
            undefined_variable_warnings(&input, &[("left", 1), ("right", 1), ("third", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
        "static PTN_UNUSED PtnValue ptn_bitwise_string_and(PtnStringOperand left, PtnStringOperand right)"
    ));
    assert!(c_source.contains(
        "static PTN_UNUSED PtnValue ptn_bitwise_string_or(PtnStringOperand left, PtnStringOperand right)"
    ));
    assert!(c_source.contains(
        "static PTN_UNUSED PtnValue ptn_bitwise_string_xor(PtnStringOperand left, PtnStringOperand right)"
    ));
    assert!(c_source.contains("size_t left_len = left.len;"));
    assert!(c_source.contains("size_t right_len = right.len;"));
    assert!(c_source.contains("ptn_bitwise_string_and(left_string, right_string)"));
    assert!(c_source.contains("ptn_bitwise_string_or(left_string, right_string)"));
    assert!(c_source.contains("ptn_bitwise_string_xor(left_string, right_string)"));
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
    assert!(c_source.contains("ptn_bitwise_string_not(string)"));
}

#[test]
fn compile_bitwise_conversion_diagnostics_respect_error_reporting_to_native_binary() {
    let root = temp_dir("ptn-native-bitwise-diagnostic-filter");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-diagnostic-filter.php");
    let output = root.join("bitwise-diagnostic-filter-bin");
    fs::write(
        &input,
        "<?php error_reporting(E_ERROR); var_dump((PHP_INT_MAX + 1) & 1); var_dump(3 | 1.5); var_dump(\"1.5abc\" ^ 0); error_reporting(E_ALL); var_dump(3 | 1.5);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "int(0)\nint(3)\nint(1)\n\nDeprecated: Implicit conversion from float 1.5 to int loses precision in ptn-generated-code on line 0\nint(3)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_bitwise_not_out_of_range_float_uses_php_warning_to_native_binary() {
    let root = temp_dir("ptn-native-bitwise-not-out-of-range-float");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("bitwise-not-out-of-range-float.php");
    let output = root.join("bitwise-not-out-of-range-float-bin");
    fs::write(&input, "<?php var_dump(~(PHP_INT_MAX + 1));").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "\nWarning: The float 9.223372036854776E+18 is not representable as an int, cast occurred in ptn on line 1\nint(9223372036854775807)\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
        "\nDeprecated: Non-canonical cast (boolean) is deprecated, use the (bool) cast instead in ptn on line 3\nbool(true)\n"
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
        "\nDeprecated: Non-canonical cast (integer) is deprecated, use the (int) cast instead in ptn on line 3\nint(42)\n\nDeprecated: Non-canonical cast (double) is deprecated, use the (float) cast instead in ptn on line 4\nfloat(42)\n\nDeprecated: Non-canonical cast (binary) is deprecated, use the (string) cast instead in ptn on line 5\nstring(2) \"42\"\n"
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
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!(
            "{}\n",
            undefined_variable_warnings(&input, &[("left", 1), ("right", 1)])
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
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
        format!(
            "defined{} done\n",
            undefined_variable_warning(&input, "missing", 1)
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn lexer_accepts_variable_variable_tokens() {
    let tokens = lexer::lex("<?php $$name; ${$name};").unwrap();
    assert!(matches!(tokens[1].kind, TokenKind::Dollar));
    assert!(matches!(&tokens[2].kind, TokenKind::Variable(name) if name == "name"));
    assert!(matches!(tokens[4].kind, TokenKind::Dollar));
    assert!(matches!(tokens[5].kind, TokenKind::LeftBrace));
    assert!(matches!(&tokens[6].kind, TokenKind::Variable(name) if name == "name"));
    assert!(matches!(tokens[7].kind, TokenKind::RightBrace));
}

#[test]
fn parser_accepts_dynamic_variable_reads_and_assignments() {
    let program = parser::parse("<?php $$name = 1; echo ${$name};").unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected dynamic variable assignment expression");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::DynamicVariable { name, .. },
            op: AssignmentOp::Assign,
            ..
        } if matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
    ));

    let Statement::Echo { expressions, .. } = &program.statements[1] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::DynamicVariable { name, .. }
            if matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
    ));
}

#[test]
fn parser_accepts_dynamic_variable_array_dimension_assignments() {
    let program = parser::parse("<?php ${$$$a}[\"key\"] = \"value\";").unwrap();
    assert_eq!(program.statements.len(), 1);

    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected dynamic array dimension assignment expression");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::DynamicArrayDim {
                name,
                dimensions,
                ..
            },
            op: AssignmentOp::Assign,
            ..
        } if dimensions.len() == 1
            && matches!(name.as_ref(), Expr::DynamicVariable { .. })
    ));
}

#[test]
fn parser_accepts_dynamic_variable_null_coalescing_assignments() {
    let program = parser::parse("<?php $$name ??= 1; ${$name}[\"key\"] ??= 2;").unwrap();
    assert_eq!(program.statements.len(), 2);

    let Statement::Expression { expression, .. } = &program.statements[0] else {
        panic!("expected dynamic variable null coalescing assignment expression");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::DynamicVariable { name, .. },
            op: AssignmentOp::CoalesceAssign,
            ..
        } if matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
    ));

    let Statement::Expression { expression, .. } = &program.statements[1] else {
        panic!("expected dynamic array dimension null coalescing assignment expression");
    };
    assert!(matches!(
        expression,
        Expr::Assign {
            target: AssignmentTarget::DynamicArrayDim {
                name,
                dimensions,
                ..
            },
            op: AssignmentOp::CoalesceAssign,
            ..
        } if dimensions.len() == 1
            && matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
    ));
}

#[test]
fn parser_accepts_dynamic_variable_array_dimension_unsets() {
    let program = parser::parse("<?php unset(${$name}[\"key\"], ${$$name}[0][\"leaf\"]);").unwrap();
    assert_eq!(program.statements.len(), 1);

    let Statement::Unset { targets, .. } = &program.statements[0] else {
        panic!("expected unset statement");
    };
    assert_eq!(targets.len(), 2);
    assert!(matches!(
        &targets[0],
        UnsetTarget::DynamicArrayDim {
            name,
            dimensions,
            ..
        } if dimensions.len() == 1
            && matches!(name.as_ref(), Expr::Variable(variable, _) if variable == "name")
            && matches!(&dimensions[0], Expr::String(value, _) if value == "key")
    ));
    assert!(matches!(
        &targets[1],
        UnsetTarget::DynamicArrayDim {
            name,
            dimensions,
            ..
        } if dimensions.len() == 2
            && matches!(name.as_ref(), Expr::DynamicVariable { .. })
    ));
}

#[test]
fn compile_dynamic_variable_reads_and_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-variable");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable.php");
    let output = root.join("dynamic-variable-bin");
    fs::write(
        &input,
        "<?php\n\
function mark($value) { echo $value, \"\\n\"; return $value; }\n\
$name = \"target\";\n\
$target = \"initial\";\n\
echo $$name, \"\\n\";\n\
$$name = \"updated\";\n\
echo $target, \"\\n\";\n\
${\"other\"} = 7;\n\
echo $other, \"\\n\";\n\
$intName = 123;\n\
${$intName} = \"number\";\n\
echo ${\"123\"}, \"\\n\";\n\
$boolName = true;\n\
${$boolName} = \"truthy\";\n\
echo ${\"1\"}, \"\\n\";\n\
$nullName = null;\n\
${$nullName} = \"empty\";\n\
echo ${\"\"}, \"\\n\";\n\
${mark(\"ordered\")} = mark(\"rhs\");\n\
echo $ordered, \"\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "initial\nupdated\n7\nnumber\ntruthy\nempty\nordered\nrhs\nrhs\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_dynamic_variable_array_dimension_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-variable-array-dim");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable-array-dim.php");
    let output = root.join("dynamic-variable-array-dim-bin");
    fs::write(
        &input,
        "<?php\n\
function mark($value) { echo $value, \"\\n\"; return $value; }\n\
$a = \"b\";\n\
$$a = \"test\";\n\
$$$a = \"blah\";\n\
${$$$a}[\"associative arrays work too\"] = \"this is nifty\";\n\
echo \"$test\\n\";\n\
echo $blah[$test = \"associative arrays work too\"], \"\\n\";\n\
$name = \"slot\";\n\
${$name}[1] = \"one\";\n\
${$name}[] = \"tail\";\n\
echo $slot[1], \":\", $slot[2], \"\\n\";\n\
$textName = \"text\";\n\
$text = \"abc\";\n\
${$textName}[1] = \"Z\";\n\
echo $text, \"\\n\";\n\
$orderedName = \"ordered\";\n\
${mark($orderedName)}[mark(\"key\")] = mark(\"value\");\n\
echo $ordered[\"key\"], \"\\n\";\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "blah\nthis is nifty\none:tail\naZc\nordered\nkey\nvalue\nvalue\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_dynamic_variable_name(&runtime"));
    assert!(c_source.contains("ptn_runtime_array_path_set(&runtime, ptn_tmp_"));
}

#[test]
fn compile_dynamic_variable_array_dimension_unsets_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-variable-array-dim-unset");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable-array-dim-unset.php");
    let output = root.join("dynamic-variable-array-dim-unset-bin");
    fs::write(
        &input,
        "<?php\n\
function mark($value) { echo $value, \"\\n\"; return $value; }\n\
$name = \"items\";\n\
$items = [\"keep\" => \"yes\", \"drop\" => \"gone\", \"nested\" => [\"x\" => 1, \"gone\" => 2]];\n\
unset(${$name}[\"drop\"], ${$name}[\"nested\"][\"gone\"]);\n\
var_dump(array_key_exists(\"drop\", $items), array_key_exists(\"gone\", $items[\"nested\"]), $items[\"keep\"], $items[\"nested\"][\"x\"]);\n\
$textName = \"text\";\n\
$text = \"abc\";\n\
try { unset(${$textName}[1]); } catch (\\Error $e) { echo $e->getMessage(), \"\\n\"; }\n\
$ordered = [\"key\" => \"value\"];\n\
unset(${mark(\"ordered\")}[mark(\"key\")]);\n\
var_dump(array_key_exists(\"key\", $ordered));\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(false)\n",
            "bool(false)\n",
            "string(3) \"yes\"\n",
            "int(1)\n",
            "Cannot unset string offsets\n",
            "ordered\n",
            "key\n",
            "bool(false)\n"
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_dynamic_variable_name(&runtime"));
    assert!(c_source.contains("ptn_runtime_array_path_unset(&runtime, ptn_tmp_"));
}

#[test]
fn compile_dynamic_variable_array_dimension_compound_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-variable-array-dim-compound");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable-array-dim-compound.php");
    let output = root.join("dynamic-variable-array-dim-compound-bin");
    fs::write(
        &input,
        "<?php\n\
function mark($value) { echo $value, \"\\n\"; return $value; }\n\
$name = \"items\";\n\
$items = [\"count\" => 1, \"text\" => \"a\", \"nested\" => [\"n\" => 2]];\n\
${$name}[\"count\"] += 4;\n\
echo $items[\"count\"], \"\\n\";\n\
echo (${$name}[\"count\"] *= 2), \":\", $items[\"count\"], \"\\n\";\n\
${mark(\"items\")}[mark(\"text\")] .= mark(\"b\");\n\
echo $items[\"text\"], \"\\n\";\n\
${$name}[\"nested\"][\"n\"] **= 3;\n\
echo $items[\"nested\"][\"n\"], \"\\n\";\n\
${$name}[mark(\"count\")] += ($items[\"count\"] = 1);\n\
echo $items[\"count\"], \"\\n\";\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "5\n10:10\nitems\ntext\nb\nab\n8\ncount\n2\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_dynamic_variable_name(&runtime"));
    assert!(c_source.contains("ptn_runtime_array_path_read_for_assign_op(&runtime, ptn_tmp_"));
    assert!(c_source.contains("ptn_runtime_array_path_set_from_assign_op(&runtime, ptn_tmp_"));
}

#[test]
fn compile_dynamic_variable_null_coalescing_assignments_to_native_binary() {
    let root = temp_dir("ptn-native-dynamic-variable-null-coalescing-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable-null-coalescing-assignment.php");
    let output = root.join("dynamic-variable-null-coalescing-assignment-bin");
    fs::write(
        &input,
        "<?php\n\
function mark($value) { echo $value, \"\\n\"; return $value; }\n\
function rhs($label) { echo \"rhs:$label\\n\"; return $label; }\n\
$name = \"slot\";\n\
var_dump($$name ??= rhs(\"missing\"));\n\
var_dump($slot);\n\
var_dump($$name ??= rhs(\"hit\"));\n\
$slot = null;\n\
var_dump($$name ??= rhs(\"null\"));\n\
$slot = false;\n\
var_dump($$name ??= rhs(\"false\"));\n\
${mark(\"ordered\")} ??= mark(\"created\");\n\
var_dump($ordered);\n\
$itemsName = \"items\";\n\
$items = [\"hit\" => \"kept\", \"nullish\" => null, \"nested\" => [\"leaf\" => null]];\n\
var_dump(${$itemsName}[\"hit\"] ??= rhs(\"array-hit\"));\n\
var_dump(${$itemsName}[\"nullish\"] ??= rhs(\"array-null\"));\n\
var_dump(${$itemsName}[\"missing\"] ??= rhs(\"array-missing\"));\n\
var_dump(${$itemsName}[\"nested\"][\"leaf\"] ??= rhs(\"nested\"));\n\
$undefName = \"undef\";\n\
var_dump(${$undefName}[\"key\"] ??= rhs(\"undef\"));\n\
$textName = \"text\";\n\
$text = \"abc\";\n\
var_dump(${$textName}[1] ??= rhs(\"string-hit\"));\n\
${mark(\"items\")}[mark(\"ordered\")] ??= mark(\"ordered-value\");\n\
var_dump($items[\"ordered\"]);\n\
var_dump($items[\"hit\"], $items[\"nullish\"], $items[\"missing\"], $items[\"nested\"][\"leaf\"], $undef[\"key\"], $text);\n",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "rhs:missing\n\
string(7) \"missing\"\n\
string(7) \"missing\"\n\
string(7) \"missing\"\n\
rhs:null\n\
string(4) \"null\"\n\
bool(false)\n\
ordered\n\
created\n\
string(7) \"created\"\n\
string(4) \"kept\"\n\
rhs:array-null\n\
string(10) \"array-null\"\n\
rhs:array-missing\n\
string(13) \"array-missing\"\n\
rhs:nested\n\
string(6) \"nested\"\n\
rhs:undef\n\
string(5) \"undef\"\n\
string(1) \"b\"\n\
items\n\
ordered\n\
ordered-value\n\
string(13) \"ordered-value\"\n\
string(4) \"kept\"\n\
string(10) \"array-null\"\n\
string(13) \"array-missing\"\n\
string(6) \"nested\"\n\
string(5) \"undef\"\n\
string(3) \"abc\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_runtime_read_variable_quiet(&runtime, ptn_tmp_"));
    assert!(c_source.contains("ptn_runtime_array_path_lookup_quiet(&runtime, ptn_tmp_"));
    assert!(c_source.contains("ptn_runtime_array_path_set(&runtime, ptn_tmp_"));
}

#[test]
fn dynamic_variable_non_scalar_names_report_diagnostic() {
    let root = temp_dir("ptn-native-dynamic-variable-unsupported-name");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-variable-unsupported-name.php");
    let output = root.join("dynamic-variable-unsupported-name-bin");
    fs::write(&input, "<?php $name = []; echo $$name;").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: Unsupported dynamic variable name of type array\n"
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
fn parser_accepts_offset_null_coalescing_assignments() {
    let program = parser::parse(
        "<?php\n\
$items[\"name\"] ??= 1;\n\
$items[\"nested\"][\"leaf\"] ??= 2;\n\
echo $items[\"expr\"] ??= 3;\n\
($items[\"grouped\"] ??= 4);\n",
    )
    .unwrap();
    assert_eq!(program.statements.len(), 4);

    let Statement::ArrayAssign { target, op, .. } = &program.statements[0] else {
        panic!("expected array null coalescing assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::CoalesceAssign);
    assert_eq!(target.dimensions.len(), 1);

    let Statement::ArrayAssign { target, op, .. } = &program.statements[1] else {
        panic!("expected nested array null coalescing assignment statement");
    };
    assert_eq!(target.array, "items");
    assert_eq!(*op, AssignmentOp::CoalesceAssign);
    assert_eq!(target.dimensions.len(), 2);

    let Statement::Echo { expressions, .. } = &program.statements[2] else {
        panic!("expected echo statement");
    };
    assert!(matches!(
        &expressions[0],
        Expr::Assign {
            target: AssignmentTarget::ArrayDim(target),
            op: AssignmentOp::CoalesceAssign,
            ..
        } if target.array == "items"
    ));

    let Statement::Expression { expression, .. } = &program.statements[3] else {
        panic!("expected grouped expression statement");
    };
    assert!(matches!(
        expression,
        Expr::Grouped { expr, .. }
            if matches!(
                expr.as_ref(),
                Expr::Assign {
                    target: AssignmentTarget::ArrayDim(target),
                    op: AssignmentOp::CoalesceAssign,
                    ..
                } if target.array == "items"
            )
    ));
}

#[test]
fn parser_rejects_append_null_coalescing_assignment_with_explicit_diagnostics() {
    let cases = [
        ("statement", "<?php\n$items[] ??= 1;"),
        ("echo expression", "<?php\necho $items[] ??= 1;"),
        ("grouped expression", "<?php\n($items[] ??= 1);"),
        ("dynamic-root expression", "<?php\n${$name}[] ??= 1;"),
    ];

    for (name, source) in cases {
        let error = parser::parse(source).unwrap_err();
        assert_eq!(
            error.message, "null coalescing assignment cannot use append array access",
            "{name}"
        );
        assert_eq!(error.kind, DiagnosticKind::Fatal, "{name}");
        let span = error.span.unwrap();
        let operator_offset = source.find("??=").unwrap();
        let before_operator = &source[..operator_offset];
        let expected_line = before_operator
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let expected_column = before_operator.rsplit('\n').next().unwrap().chars().count() + 1;
        assert_eq!(span.byte_start, operator_offset, "{name}");
        assert_eq!(span.byte_end, operator_offset + 3, "{name}");
        assert_eq!(span.line, expected_line, "{name}");
        assert_eq!(span.column, expected_column, "{name}");
    }
}

#[test]
fn phpc_renders_append_null_coalescing_assignment_diagnostic() {
    let root = temp_dir("ptn-phpc-append-null-coalescing-assignment");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("append-null-coalescing-assignment.php");
    fs::write(&input, "<?php\n$items[] ??= \"fallback\";\n").unwrap();

    let execution = Command::new(phpc_bin()).arg(&input).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(execution.status.code(), Some(255));
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        format!(
            "Fatal error: null coalescing assignment cannot use append array access in {} on line 2\n",
            input.display()
        )
    );
}

#[test]
fn var_dump_complex_edges_remain_unsupported_before_codegen() {
    let source = "<?php $array = []; $array[] = &$array; var_dump($array);";
    assert!(
        parser::parse(source).is_err(),
        "expected unsupported recursive array var_dump edge to fail before codegen"
    );
}

#[test]
fn compile_var_dump_stdclass_properties_to_native_binary() {
    let root = temp_dir("ptn-native-var-dump-stdclass");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("var-dump-stdclass.php");
    let output = root.join("var-dump-stdclass-bin");
    fs::write(
        &input,
        "<?php
$object = new stdClass;
$object->value = \"visible\";
var_dump($object);
",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "object(stdClass)#1 (1) {\n  [\"value\"]=>\n  string(7) \"visible\"\n}\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_declared_non_public_property_metadata_to_native_binary() {
    let root = temp_dir("ptn-native-non-public-property-metadata");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("non-public-property-metadata.php");
    let output = root.join("non-public-property-metadata-bin");
    fs::write(
        &input,
        "<?php
class Secret {
    private $hidden;
    protected $shared = 8;
    public $visible = \"ok\";
    function __construct($value) {
        $this->hidden = $value;
    }
    public function expose() {
        return $this->hidden . \":\" . $this->shared . \":\" . $this->visible;
    }
}
$object = new Secret(7);
echo $object->expose(), \"\\n\";
var_dump($object);
echo var_export($object, true), \"\\n\";
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "7:8:ok\n",
            "object(Secret)#1 (3) {\n",
            "  [\"hidden\":\"Secret\":private]=>\n",
            "  int(7)\n",
            "  [\"shared\":protected]=>\n",
            "  int(8)\n",
            "  [\"visible\"]=>\n",
            "  string(2) \"ok\"\n",
            "}\n",
            "\\Secret::__set_state(array(\n",
            "   'hidden' => 7,\n",
            "   'shared' => 8,\n",
            "   'visible' => 'ok',\n",
            "))\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_declare_property(&runtime"));
    assert!(c_source.contains("PTN_PROPERTY_PRIVATE"));
    assert!(c_source.contains("PTN_PROPERTY_PROTECTED"));
}

#[test]
fn compile_non_public_property_static_compare_and_export_to_native_binary() {
    let root = temp_dir("ptn-native-non-public-property-static-compare");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("non-public-property-static-compare.php");
    let output = root.join("non-public-property-static-compare-bin");
    fs::write(
        &input,
        "<?php
class Secret {
    private $value;
    protected $token = \"seed\";
    public $label;

    public function __construct($value) {
        $this->value = $value;
        $this->label = \"box\";
    }

    public static function compare($left, $right) {
        return $left->value === $right->value;
    }

    public function update($value) {
        $this->token = $value;
        return $this->token;
    }
}

$a = new Secret(4);
$b = new Secret(4);
var_dump(Secret::compare($a, $b));
var_dump($a->update(\"changed\"));
var_dump($a);
echo var_export($a, true), \"\\n\";
try {
    var_dump($a->value);
} catch (Error $e) {
    echo $e->getMessage(), \"\\n\";
}
",
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "bool(true)\n",
            "string(7) \"changed\"\n",
            "object(Secret)#1 (3) {\n",
            "  [\"value\":\"Secret\":private]=>\n",
            "  int(4)\n",
            "  [\"token\":protected]=>\n",
            "  string(7) \"changed\"\n",
            "  [\"label\"]=>\n",
            "  string(3) \"box\"\n",
            "}\n",
            "\\Secret::__set_state(array(\n",
            "   'value' => 4,\n",
            "   'token' => 'changed',\n",
            "   'label' => 'box',\n",
            "))\n",
            "Cannot access private property Secret::$value\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_object_declare_property(&runtime"));
    assert!(c_source.contains("PTN_PROPERTY_PRIVATE"));
    assert!(c_source.contains("PTN_PROPERTY_PROTECTED"));
}

#[test]
fn compile_stream_resources_to_native_binary() {
    let root = temp_dir("ptn-native-stream-resources");
    fs::create_dir_all(&root).unwrap();
    let data = root.join("payload.txt");
    fs::write(&data, "payload").unwrap();
    let input = root.join("stream-resources.php");
    let output = root.join("stream-resources-bin");
    let data_path = data.to_string_lossy();
    fs::write(
        &input,
        format!(
            "<?php\n\
$fp = fopen(\"{}\", \"r\");\n\
var_dump(gettype($fp), is_resource($fp), function_exists(\"fopen\"), function_exists(\"fclose\"), function_exists(\"is_resource\"));\n\
var_dump($fp);\n\
var_dump(array_key_exists($fp, [2 => \"no\"]));\n\
var_dump(fclose($fp), is_resource($fp), gettype($fp));",
            data_path
        ),
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "string(8) \"resource\"\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
resource(5) of type (stream)\n\
\n\
Warning: Resource ID#5 used as offset, casting to integer (5) in ptn on line 5\n\
bool(false)\n\
bool(true)\n\
bool(false)\n\
string(17) \"resource (closed)\"\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_stream_get_meta_data_to_native_binary() {
    let root = temp_dir("ptn-native-stream-get-meta-data");
    fs::create_dir_all(&root).unwrap();
    let data = root.join("payload.txt");
    fs::write(&data, "payload").unwrap();
    let input = root.join("stream-get-meta-data.php");
    let output = root.join("stream-get-meta-data-bin");
    let data_path = data.to_string_lossy();
    fs::write(
        &input,
        format!(
            "<?php\n\
$fp = fopen(\"{}\", \"r\");\n\
$meta = stream_get_meta_data($fp);\n\
var_dump(function_exists(\"stream_get_meta_data\"));\n\
var_dump($meta[\"timed_out\"], $meta[\"blocked\"], $meta[\"eof\"]);\n\
var_dump($meta[\"wrapper_type\"], $meta[\"stream_type\"], $meta[\"mode\"]);\n\
var_dump($meta[\"unread_bytes\"], $meta[\"seekable\"], str_contains($meta[\"uri\"], \"payload.txt\"));\n\
var_dump(fclose($fp));\n\
try {{\n\
    stream_get_meta_data($fp);\n\
}} catch (TypeError $e) {{\n\
    echo $e->getMessage(), \"\\n\";\n\
}}",
            data_path
        ),
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "bool(true)\n\
bool(false)\n\
bool(true)\n\
bool(false)\n\
string(9) \"plainfile\"\n\
string(5) \"STDIO\"\n\
string(1) \"r\"\n\
int(0)\n\
bool(true)\n\
bool(true)\n\
bool(true)\n\
stream_get_meta_data(): Argument #1 ($stream) must be an open stream resource\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_namespaced_class_aliases_to_native_binary() {
    let root = temp_dir("ptn-native-namespace-class-aliases");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("namespace-class-aliases.php");
    let output = root.join("namespace-class-aliases-bin");
    fs::write(
        &input,
        r#"<?php
namespace test\ns1;

class Foo {
    function __construct() { echo __CLASS__, "\n"; }
    function bar() { echo __CLASS__, "\n"; }
    static function baz() { echo __CLASS__, "\n"; }
}

use test\ns1\Foo as Bar;
use test\ns1 as ns2;
use test\ns1;

$x = new Foo;
$x->bar();
Foo::baz();
$y = new \test\ns1\Foo;
$y->bar();
\test\ns1\Foo::baz();
Bar::baz();
ns2\Foo::baz();
ns1\Foo::baz();
"#,
    )
    .unwrap();

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
            "test\\ns1\\Foo\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("test\\\\ns1\\\\Foo"));
}

#[test]
fn compile_namespaced_function_and_constant_imports_to_native_binary() {
    let root = temp_dir("ptn-native-namespace-function-constant-imports");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("namespace-function-constant-imports.php");
    let output = root.join("namespace-function-constant-imports-bin");
    fs::write(
        &input,
        r#"<?php
namespace Lib\Tools;

const MARK = "const\n";

function mark($value) {
    return __NAMESPACE__ . ":" . $value . "\n";
}

namespace App;

use function Lib\Tools\mark as label;
use const Lib\Tools\MARK as IMPORTED_MARK;

echo label("call");
echo IMPORTED_MARK;
echo strlen("abc"), ":", PHP_INT_SIZE, "\n";
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        "Lib\\Tools:call\nconst\n3:8\n"
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn parser_rejects_namespace_after_function_declaration() {
    let error = parser::parse("<?php function foo() {} namespace Bar;").unwrap_err();
    assert_eq!(
        error.message,
        "Namespace declaration statement has to be the very first statement or after any declare call in the script"
    );
}

#[test]
fn unsupported_internal_functions_fail_in_generated_runtime() {
    let root = temp_dir("ptn-native-unsupported-internal-function");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("unsupported-internal-function.php");
    let output = root.join("unsupported-internal-function-bin");
    fs::write(&input, "<?php definitely_missing_internal();").unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(!execution.status.success());
    assert_eq!(String::from_utf8(execution.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(execution.stderr).unwrap(),
        "Fatal error: Call to undefined function definitely_missing_internal()\n"
    );
}

#[test]
fn support_docs_name_var_dump_unsupported_edges() {
    let support = fs::read_to_string("docs/SUPPORT.md").unwrap();
    assert!(support.contains("Array read expressions"));
    assert!(support.contains("String offset read expressions"));
    assert!(support.contains("complete reference identity"));
    assert!(support.contains("String offset writes/mutation"));
    assert!(support.contains("recursive arrays"));
    assert!(support.contains("objects"));
    assert!(support.contains("resources"));
    assert!(support.contains("references"));
    assert!(support.contains("Embedded NUL strings"));
    assert!(support.contains("Remaining PHP float precision and formatting edge cases"));
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
