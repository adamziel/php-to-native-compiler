use php_compiler::run_source;

#[test]
fn similar_text_counts_and_writes_percent_to_direct_variable() {
    let execution = run_source(
        r#"<?php
var_dump(similar_text("abcdefgh", "efg"));
var_dump(similar_text("abcdefgh", "mno"));
var_dump(similar_text("abcdefghcc", "c"));
var_dump(similar_text("abcdefghabcdef", "zzzzabcdefggg"));
$percent = 0;
similar_text("abcdefgh", "efg", $percent);
var_dump($percent);
similar_text("abcdefgh", "mno", $percent);
var_dump($percent);
similar_text("abcdefghcc", "c", $percent);
var_dump($percent);
similar_text("abcdefghabcdef", "zzzzabcdefggg", $percent);
var_dump($percent);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(3)\n",
            "int(0)\n",
            "int(1)\n",
            "int(7)\n",
            "float(54.54545454545455)\n",
            "float(0)\n",
            "float(18.181818181818183)\n",
            "float(51.851851851851855)\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_word_count_counts_words_offsets_and_extra_characters() {
    let execution = run_source(
        r#"<?php
$str = "Hello friend, you're
    looking          good today!";
$words = str_word_count($str, 2);
echo count($words), "|", $words[0], "|", $words[14], "|", $words[47], "\n";
echo str_word_count($str), "\n";
$str2 = "F0o B4r 1s bar foo";
echo str_word_count($str2, 0, "04"), "|";
echo str_word_count($str2, 0, "01"), "|";
echo str_word_count($str2, 0, "014"), "|";
echo str_word_count($str2, 0, ""), "\n";
var_dump(str_word_count("foo'0 bar-0var", 2, "0"));
var_dump(str_word_count("'foo'", 2));
var_dump(str_word_count("'foo'", 2, "'"));
var_dump(str_word_count("-foo-", 2));
var_dump(str_word_count("-foo-", 2, "-"));
try {
    str_word_count($str, 3);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "6|Hello|you're|today\n",
            "6\n",
            "5|6|5|7\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  string(5) \"foo'0\"\n",
            "  [6]=>\n",
            "  string(8) \"bar-0var\"\n",
            "}\n",
            "array(1) {\n",
            "  [1]=>\n",
            "  string(4) \"foo'\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(5) \"'foo'\"\n",
            "}\n",
            "array(1) {\n",
            "  [1]=>\n",
            "  string(3) \"foo\"\n",
            "}\n",
            "array(1) {\n",
            "  [0]=>\n",
            "  string(5) \"-foo-\"\n",
            "}\n",
            "str_word_count(): Argument #2 ($format) must be a valid format value\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_introspection_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["similar_text", "str_word_count"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
$percent = (new ReflectionFunction("similar_text"))->getParameters()[2];
echo $percent->isPassedByReference() ? "ref" : "value";
$call = "similar_text";
$dynamic_percent = 0;
echo "|", $call("abc", "abc", $dynamic_percent), ":", ($dynamic_percent > 99 ? "pct" : "bad");
$words = "str_word_count";
echo "|", $words("one two");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:2/3;11:1/3;ref|3:pct|2");
    assert_eq!(execution.exit_code, 0);
}
