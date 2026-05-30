use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn trim_executes_current_default_mask_subset() {
    let execution = run_source(
        "<?php\n\
echo trim(\" \\t128M\\n\"), \"|\";\n\
echo trim(\"\\tabc\\n\"), \"|\";\n\
echo trim(null), \"|\";\n\
echo trim(42);\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|abc||42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trim_executes_custom_mask_ranges_and_empty_masks() {
    let execution = run_source(
        "<?php\n\
echo trim(\"9.alpha0\", \"0..9.\"), \"|\";\n\
echo trim(\"AZpayloadaz\", \"A..Zaz\"), \"|\";\n\
echo trim(\" unchanged \", \"\");\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "alpha|payload| unchanged ");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trim_is_available_through_string_valued_calls() {
    let execution = run_source(
        "<?php\n\
$call = \"trim\";\n\
echo function_exists($call) ? \"yes\" : \"no\";\n\
echo \"|\";\n\
echo is_callable($call) ? \"callable\" : \"missing\";\n\
echo \"|\";\n\
echo $call(\" ABC \");\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|ABC");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ltrim_executes_current_default_and_slash_mask_subset() {
    let execution = run_source(
        "<?php\n\
echo ltrim(\" \\t128M\\n\"), \"|\";\n\
echo ltrim(\"///wp-content\", \"/\"), \"|\";\n\
echo ltrim(\"\\r\\n\\t (SELECT\", \"\\r\\n\\t (\"), \"|\";\n\
echo ltrim(\"AZpayload\", \"A..Z\"), \"|\";\n\
echo ltrim(null), \"|\";\n\
$call = \"ltrim\";\n\
echo $call(\"//plugins\", \"/\");\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "128M\n|wp-content|SELECT|payload||plugins"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn rtrim_executes_current_default_and_literal_mask_subset() {
    let execution = run_source(
        "<?php\n\
echo rtrim(\" \\t128M\\n\"), \"|\";\n\
echo rtrim(\"localhost/\", \"/\"), \"|\";\n\
echo rtrim(\"/wp-admin///\", \"/\"), \"|\";\n\
echo rtrim(\"PAYLOADaz\", \"a..z\"), \"|\";\n\
echo rtrim(null), \"|\";\n\
$call = \"rtrim\";\n\
echo $call(\"example.test///\", \"/\");\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        " \t128M|localhost|/wp-admin|PAYLOAD||example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chop_alias_executes_rtrim_semantics_and_metadata() {
    let execution = run_source(
        "<?php\n\
echo function_exists(\"chop\") ? \"yes\" : \"no\";\n\
echo \"|\";\n\
echo is_callable(\"chop\") ? \"callable\" : \"missing\";\n\
echo \"|\";\n\
echo chop(\"hello world\\t\\n\\r\\0\\x0B  \"), \"|\";\n\
echo chop(\"hello123\", \"0..9\"), \"|\";\n\
$call = \"chop\";\n\
echo $call(\"example.test///\", \"/\");\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|hello world|hello|example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trim_rejects_forms_outside_supported_mask_semantics() {
    let array_arg = run_source("<?php\ntrim(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call trim(): arrays are not supported"
    );

    let ambiguous = run_source("<?php\ntrim('ABC', 'A...Z');\n").unwrap_err();
    assert_eq!(ambiguous.phase, Phase::Runtime);
    assert_eq!(ambiguous.line, 2);
    assert_eq!(ambiguous.column, 1);
    assert_eq!(
        ambiguous.message,
        "unsupported call trim(): character mask ranges are not fully implemented: ambiguous dot-runs are blocked until full PHP charlist parsing is implemented"
    );

    let too_few = run_source("<?php\ntrim();\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for trim(): expected 1 to 2 argument(s), got 0"
    );
}

#[test]
fn ltrim_rejects_forms_outside_supported_mask_semantics() {
    let array_arg = run_source("<?php\nltrim(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call ltrim(): arrays are not supported"
    );

    let mask_array = run_source("<?php\nltrim('ABC', ['A']);\n").unwrap_err();
    assert_eq!(mask_array.phase, Phase::Runtime);
    assert_eq!(mask_array.line, 2);
    assert_eq!(mask_array.column, 1);
    assert_eq!(
        mask_array.message,
        "unsupported call ltrim(): character mask arrays are not supported"
    );

    let range_mask = run_source("<?php\nltrim('ABC', '..Z');\n").unwrap_err();
    assert_eq!(range_mask.phase, Phase::Runtime);
    assert_eq!(range_mask.line, 2);
    assert_eq!(range_mask.column, 1);
    assert_eq!(
        range_mask.message,
        "unsupported call ltrim(): character mask ranges are not fully implemented: no character to the left of '..'"
    );

    let too_few = run_source("<?php\nltrim();\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for ltrim(): expected 1 to 2 argument(s), got 0"
    );
}

#[test]
fn rtrim_rejects_forms_outside_supported_mask_semantics() {
    let array_arg = run_source("<?php\nrtrim(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call rtrim(): arrays are not supported"
    );

    let mask_array = run_source("<?php\nrtrim('ABC', ['C']);\n").unwrap_err();
    assert_eq!(mask_array.phase, Phase::Runtime);
    assert_eq!(mask_array.line, 2);
    assert_eq!(mask_array.column, 1);
    assert_eq!(
        mask_array.message,
        "unsupported call rtrim(): character mask arrays are not supported"
    );

    let range_mask = run_source("<?php\nrtrim('ABC', 'Z..A');\n").unwrap_err();
    assert_eq!(range_mask.phase, Phase::Runtime);
    assert_eq!(range_mask.line, 2);
    assert_eq!(range_mask.column, 1);
    assert_eq!(
        range_mask.message,
        "unsupported call rtrim(): character mask ranges are not fully implemented: '..'-ranges must be incrementing"
    );

    let too_few = run_source("<?php\nrtrim();\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for rtrim(): expected 1 to 2 argument(s), got 0"
    );
}

#[test]
fn emit_ir_folds_trim_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        "<?php\n\
echo function_exists(\"trim\") ? \"1\" : \"0\";\n\
echo is_callable(\"trim\") ? \"1\" : \"0\";\n",
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\ntrim(' ABC ');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_ltrim_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        "<?php\n\
echo function_exists(\"ltrim\") ? \"1\" : \"0\";\n\
echo is_callable(\"ltrim\") ? \"1\" : \"0\";\n",
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nltrim('/wp', '/');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_rtrim_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        "<?php\n\
echo function_exists(\"rtrim\") ? \"1\" : \"0\";\n\
echo is_callable(\"rtrim\") ? \"1\" : \"0\";\n",
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nrtrim('/wp/', '/');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_chop_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        "<?php\n\
echo function_exists(\"chop\") ? \"1\" : \"0\";\n\
echo is_callable(\"chop\") ? \"1\" : \"0\";\n",
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nchop('/wp/', '/');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
