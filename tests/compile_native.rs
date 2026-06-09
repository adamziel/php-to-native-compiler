use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::ast::{BinaryOp, Expr, Statement};
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
fn parser_rejects_print_expression_contexts() {
    let error = parser::parse("<?php $result = print \"hello\";").unwrap_err();
    assert!(error.message.contains("expected expression"));
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
    let error = parser::parse("<?php $name += 1;").unwrap_err();
    assert!(error.message.contains("expected assignment"));
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
