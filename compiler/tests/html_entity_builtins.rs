use php_compiler::{emit_ir_source, run_source};

#[test]
fn htmlspecialchars_and_decode_support_quote_flags_and_double_encode() {
    let execution = run_source(
        r#"<?php
$input = "Roy's <tag> & \"quote\"";
echo htmlspecialchars($input), "\n";
echo htmlspecialchars($input, ENT_NOQUOTES), "\n";
echo htmlspecialchars("&quot;&amp;xyz&gt;", ENT_NOQUOTES, "UTF-8", false), "\n";
echo htmlspecialchars_decode("Roy&#039;s &lt;tag&gt; &amp; &quot;quote&quot;"), "\n";
echo htmlspecialchars_decode("Roy&#039;s &quot;x&quot;", ENT_COMPAT), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Roy&#039;s &lt;tag&gt; &amp; &quot;quote&quot;\nRoy's &lt;tag&gt; &amp; \"quote\"\n&quot;&amp;xyz&gt;\nRoy's <tag> & \"quote\"\nRoy&#039;s \"x\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn htmlentities_and_entity_decode_cover_core_entities_and_selected_latin1() {
    let execution = run_source(
        r#"<?php
$input = "<>\"&åÄ";
echo htmlentities($input, ENT_COMPAT, "UTF-8"), "\n";
echo html_entity_decode("&lt;&gt;&quot;&amp;&aring;&Auml;", ENT_COMPAT, "UTF-8"), "\n";
echo html_entity_decode("&amp;lt;", ENT_COMPAT, "koi8-r"), "\n";
echo html_entity_decode("&#x24; &#36; &apos;", ENT_QUOTES | ENT_HTML5, "UTF-8"), "\n";
echo html_entity_decode("&apos;", ENT_QUOTES | ENT_HTML401, "UTF-8"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "&lt;&gt;&quot;&amp;&aring;&Auml;\n<>\"&åÄ\n&lt;\n$ $ '\n&apos;\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn html_entity_decode_respects_selected_iso_8859_1_entities() {
    let execution = run_source(
        r#"<?php
var_dump(bin2hex(html_entity_decode("&#233;", ENT_QUOTES, "ISO-8859-1")));
var_dump(bin2hex(html_entity_decode("&eacute;", ENT_QUOTES, "ISO-8859-1")));
echo html_entity_decode("&OElig;", ENT_NOQUOTES, "ISO-8859-1"), "\n";
echo html_entity_decode("&quot;|&#34;|&quot;|&#34;", ENT_NOQUOTES, "UTF-8"), "\n";
echo html_entity_decode("&quot;|&#34;|&quot;|&#34;", ENT_QUOTES, "UTF-8"), "\n";
echo html_entity_decode("&#39;|&#39;", ENT_NOQUOTES, "UTF-8"), "\n";
echo html_entity_decode("&#39;|&#39;", ENT_QUOTES, "UTF-8"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(2) \"e9\"\nstring(2) \"e9\"\n&OElig;\n&quot;|&#34;|&quot;|&#34;\n\"|\"|\"|\"\n&#39;|&#39;\n'|'\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn htmlspecialchars_reports_unsupported_charsets_and_uses_utf8_fallback() {
    let execution = run_source(
        r#"<?php
var_dump(htmlspecialchars("<>", ENT_COMPAT, 1));
var_dump(htmlspecialchars("<>", ENT_COMPAT, 1252));
var_dump(htmlspecialchars("<>", ENT_COMPAT, 866));
var_dump(htmlspecialchars("<>", ENT_COMPAT, "SJIS"));
var_dump(htmlspecialchars("<>", ENT_COMPAT, str_repeat("a", 12)));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Warning: htmlspecialchars(): Charset \"1\" is not supported, assuming UTF-8 in Command line code on line 2\n",
            "string(8) \"&lt;&gt;\"\n",
            "string(8) \"&lt;&gt;\"\n",
            "string(8) \"&lt;&gt;\"\n",
            "string(8) \"&lt;&gt;\"\n",
            "\nWarning: htmlspecialchars(): Charset \"aaaaaaaaaaaa\" is not supported, assuming UTF-8 in Command line code on line 6\n",
            "string(8) \"&lt;&gt;\"\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn html_translation_table_metadata_and_constants_are_available() {
    let execution = run_source(
        r#"<?php
$special = get_html_translation_table(HTML_SPECIALCHARS, ENT_QUOTES, "UTF-8");
echo $special["&"], "|", $special["<"], "|", $special["\""], "|", $special["'"], "\n";
$entities = get_html_translation_table(HTML_ENTITIES, ENT_COMPAT, "UTF-8");
echo $entities["å"], "|", $entities["Ä"], "\n";
echo function_exists("htmlspecialchars") ? "fn" : "missing";
echo "|", is_callable("html_entity_decode") ? "callable" : "not";
echo "|", ENT_QUOTES, ":", ENT_SUBSTITUTE, ":", ENT_HTML5;
echo "|";
$function = new ReflectionFunction("get_html_translation_table");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "&amp;|&lt;|&quot;|&#039;\n&aring;|&Auml;\nfn|callable|3:8:48|get_html_translation_table:0/3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn html_reflection_function_strings_match_selected_php_metadata() {
    let execution = run_source(
        r#"<?php
echo new ReflectionFunction('htmlspecialchars'), "\n";
echo new ReflectionFunction('get_html_translation_table'), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Function [ <internal:standard> function htmlspecialchars ] {\n",
            "\n",
            "  - Parameters [4] {\n",
            "    Parameter #0 [ <required> string $string ]\n",
            "    Parameter #1 [ <optional> int $flags = ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401 ]\n",
            "    Parameter #2 [ <optional> ?string $encoding = null ]\n",
            "    Parameter #3 [ <optional> bool $double_encode = true ]\n",
            "  }\n",
            "  - Return [ string ]\n",
            "}\n",
            "\n",
            "Function [ <internal:standard> function get_html_translation_table ] {\n",
            "\n",
            "  - Parameters [3] {\n",
            "    Parameter #0 [ <optional> int $table = HTML_SPECIALCHARS ]\n",
            "    Parameter #1 [ <optional> int $flags = ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401 ]\n",
            "    Parameter #2 [ <optional> string $encoding = \"UTF-8\" ]\n",
            "  }\n",
            "  - Return [ array ]\n",
            "}\n",
            "\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_html_entity_builtin_metadata_and_constants() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("htmlspecialchars") ? "1" : "0";
echo is_callable("html_entity_decode") ? "1" : "0";
echo defined("HTML_SPECIALCHARS") ? "1" : "0";
echo defined("HTML_ENTITIES") ? "1" : "0";
echo defined("ENT_QUOTES") ? "1" : "0";
echo defined("ENT_HTML5") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("HTML_SPECIALCHARS"), "{ir}");
    assert!(!ir.contains("ENT_QUOTES"), "{ir}");
}
