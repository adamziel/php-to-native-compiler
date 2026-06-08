use php_compiler::error::Phase;
use php_compiler::{parse, run_source};

#[test]
fn numeric_literal_separators_are_normalized_for_supported_radices() {
    let execution = run_source(
        r#"<?php
var_dump(299_792_458 === 299792458);
var_dump(135_00 === 13500);
var_dump(96_485.332_12 === 96485.33212);
var_dump(6.626_070_15e-34 === 6.62607015e-34);
var_dump(6.674_083e-11 === 6.674083e-11);
var_dump(0xCAFE_F00D === 0xCAFEF00D);
var_dump(0b0101_1111 === 0b01011111);
var_dump(0137_041 === 0137041);
var_dump(0_124 === 0124);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn invalid_numeric_separator_positions_remain_parser_errors() {
    for source in [
        "<?php\n100_;\n",
        "<?php\n10__0;\n",
        "<?php\n100_.0;\n",
        "<?php\n100._0;\n",
        "<?php\n0x_0123;\n",
        "<?php\n0b_0101;\n",
        "<?php\n1_e2;\n",
        "<?php\n1e_2;\n",
    ] {
        let error = parse(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert!(error
            .message
            .starts_with("syntax error, unexpected identifier"));
    }
}
