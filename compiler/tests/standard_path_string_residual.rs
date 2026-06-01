use php_compiler::run_source;

#[test]
fn metaphone_and_hebrev_cover_standard_string_residual_phpt_rows() {
    let execution = run_source(
        r#"<?php
var_dump(metaphone(""));
var_dump(metaphone(-1));
try {
    metaphone("valid phrase", -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(metaphone("valid phrase", 0));
var_dump(metaphone("They fell forward, grovelling heedlessly on the cold earth."));
echo metaphone("CMXFXZ"), "|", metaphone("CMXFXV"), "|", metaphone("CMXFXZXZ"), "\n";
echo metaphone("scratch"), "|", metaphone("scrath"), "|", metaphone("scratc"), "\n";
echo metaphone("kn"), "|", metaphone("gn"), "|", metaphone("pn"), "|", metaphone("ae"), "|", metaphone("wr"), "|", metaphone("x"), "|", metaphone("wh"), "|", metaphone("wa"), "\n";

$hebrew_text = "The hebrev function converts logical Hebrew text to visual text.\nThe function tries to avoid breaking words.\n";
var_dump(hebrev($hebrew_text));
var_dump(hebrev($hebrew_text, 15));
echo function_exists("metaphone") ? "metaphone" : "missing";
echo "|";
echo function_exists("hebrev") ? "hebrev" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(0) \"\"\n\
string(0) \"\"\n\
metaphone(): Argument #2 ($max_phonemes) must be greater than or equal to 0\n\
string(6) \"FLTFRS\"\n\
string(26) \"0FLFRWRTKRFLNKHTLSLN0KLTR0\"\n\
KMKSFKSS|KMKSFKSF|KMKSFKSSKSS\n\
SKRX|SKR0|SKRTK\n\
N|N|N|E|R|S|W|W\n\
string(109) \".The hebrev function converts logical Hebrew text to visual text\n\
.The function tries to avoid breaking words\n\
\"\n\
string(109) \"to visual text\n\
Hebrew text\n\
logical\n\
converts\n\
hebrev function\n\
.The\n\
breaking words\n\
tries to avoid\n\
.The function\n\
\"\n\
metaphone|hebrev"
    );
    assert_eq!(execution.exit_code, 0);
}
