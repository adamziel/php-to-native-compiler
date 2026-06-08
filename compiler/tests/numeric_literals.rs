use php_compiler::error::Phase;
use php_compiler::{parse, run_source};

#[test]
fn numeric_literal_separators_match_php_literal_values() {
    let execution = run_source(
        r#"<?php
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
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(true)\n".repeat(11));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn binary_and_explicit_octal_literals_execute_in_current_int_subset() {
    let execution = run_source(
        r#"<?php
echo 0b1010, "|", 0B11, "|", 0o16, "|", 0O16;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10|3|14|14");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn invalid_numeric_separator_boundaries_remain_parse_errors() {
    for source in [
        "<?php\n100_;\n",
        "<?php\n10__0;\n",
        "<?php\n0x0__F;\n",
        "<?php\n0b0__1;\n",
        "<?php\n1_e2;\n",
        "<?php\n1e_2;\n",
    ] {
        let error = parse(source).expect_err("invalid separator shape should not parse");
        assert_eq!(error.phase, Phase::Parse, "{source}");
    }
}
