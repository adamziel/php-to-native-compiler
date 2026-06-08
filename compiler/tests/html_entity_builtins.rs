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
fn html_entities_cover_html4_tables_and_single_byte_charsets() {
    let execution = run_source(
        r#"<?php
echo count(get_html_translation_table(HTML_ENTITIES, ENT_COMPAT, "UTF-8")), "\n";
echo count(get_html_translation_table(HTML_ENTITIES, ENT_QUOTES | ENT_XML1, "UTF-8")), "\n";
echo count(get_html_translation_table(HTML_ENTITIES, ENT_QUOTES | ENT_HTML401, "SJIS")), "\n";
echo get_html_translation_table(HTML_ENTITIES, ENT_QUOTES | ENT_XHTML, "UTF-8")["'"], "\n";
echo htmlentities("\x82\x86\x99\x9f", ENT_QUOTES, "Windows-1252"), "\n";
echo htmlentities("\xa4\xa6\xa8\xb4\xbc", ENT_QUOTES, "ISO-8859-15"), "\n";
echo bin2hex(html_entity_decode("&euro;&trade;&Yuml;", ENT_QUOTES, "Windows-1252")), "\n";
echo html_entity_decode("&apos;&notin;", ENT_QUOTES | ENT_XHTML, "UTF-8"), "\n";
echo html_entity_decode("&#x20AC;&#x2019;", ENT_QUOTES, "Windows-1252") === "\x80\x92" ? "win1252" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "252\n5\n5\n&apos;\n&sbquo;&dagger;&trade;&Yuml;\n&euro;&Scaron;&scaron;&Zcaron;&OElig;\n80999f\n'∉\nwin1252"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn html_entities_use_internal_encoding_for_explicit_empty_encoding() {
    let execution = run_source(
        r#"<?php
ini_set("internal_encoding", "cp1252");
echo mb_internal_encoding(), "\n";
echo htmlentities("\x82\x86\x99\x9f", ENT_QUOTES, ""), "\n";
ini_set("internal_encoding", "ISO-8859-15");
echo htmlentities("\xbc\xbd\xbe", ENT_QUOTES, ""), "\n";
ini_set("internal_encoding", "EUC-JP");
var_dump(htmlentities("\xa1\xa2\xa1\xa3\xa1\xa4", ENT_QUOTES, ""));
ini_set("internal_encoding", "");
ini_set("default_charset", "Shift_JIS");
var_dump(bin2hex(htmlentities("\x81\x41\x81\x42\x81\x43", ENT_QUOTES, "")));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Windows-1252\n",
            "&sbquo;&dagger;&trade;&Yuml;\n",
            "&OElig;&oelig;&Yuml;\n",
            "\nNotice: htmlentities(): Only basic entities substitution is supported for multi-byte encodings other than UTF-8; functionality is equivalent to htmlspecialchars in Command line code on line 8\n",
            "string(6) \"������\"\n",
            "\nNotice: htmlentities(): Only basic entities substitution is supported for multi-byte encodings other than UTF-8; functionality is equivalent to htmlspecialchars in Command line code on line 11\n",
            "string(12) \"814181428143\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn htmlentities_respects_quote_modes_and_invalid_utf8_flags() {
    let execution = run_source(
        r#"<?php
echo htmlentities("'", ENT_NOQUOTES, "UTF-8"), "\n";
echo htmlentities("'", ENT_COMPAT, "UTF-8"), "\n";
echo htmlentities("'", ENT_QUOTES, "UTF-8"), "\n";
echo htmlentities("\x80", ENT_QUOTES, "UTF-8") === "" ? "empty" : "bad", "\n";
echo bin2hex(htmlentities("\x80", ENT_QUOTES | ENT_SUBSTITUTE, "UTF-8")), "\n";
echo htmlentities("\x80", ENT_QUOTES | ENT_IGNORE, "UTF-8") === "" ? "ignore" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "'\n'\n&#039;\nempty\nefbfbd\nignore");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn html_entities_respect_invalid_entity_and_disallowed_codepoint_boundaries() {
    let execution = run_source(
        r#"<?php
echo htmlentities("&9; &kff;", ENT_QUOTES, "UTF-8", false), "\n";
echo bin2hex(htmlspecialchars("\xE3\x80\"", ENT_QUOTES | ENT_SUBSTITUTE, "UTF-8")), "\n";
echo html_entity_decode("&#x7F;", ENT_QUOTES | ENT_HTML401, "UTF-8"), "\n";
echo html_entity_decode("&#x0C;", ENT_QUOTES | ENT_HTML5, "UTF-8") === "\x0c" ? "form-feed\n" : "bad\n";
echo bin2hex(htmlentities("\x00", ENT_HTML401 | ENT_DISALLOWED, "UTF-8")), "\n";
echo htmlentities("\x09", ENT_HTML5 | ENT_DISALLOWED, "UTF-8"), "\n";
echo bin2hex(htmlentities("\xef\xbf\xbe", ENT_HTML5 | ENT_DISALLOWED, "UTF-8")), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "&amp;9; &amp;kff;\nefbfbd2671756f743b\n&#x7F;\nform-feed\nefbfbd\n&Tab;\nefbfbd\n"
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
