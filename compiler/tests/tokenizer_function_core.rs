use php_compiler::error::Phase;
use php_compiler::php_tokenizer::{self, PhpTokenizerToken};
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn token_text(token: &PhpTokenizerToken) -> String {
    String::from_utf8_lossy(token.text()).into_owned()
}

#[test]
fn tokenizer_scans_core_stream_shape_namespaces_attributes_and_lines() {
    let tokens = php_tokenizer::tokenize(
        b"before <?php #[Attr]\nnamespace Foo\\Bar; echo $name; // hi\n?>after",
    );

    assert!(
        tokens.iter().any(|token| {
            token.id() == php_tokenizer::T_INLINE_HTML && token_text(token) == "before "
        }),
        "expected leading inline HTML token"
    );
    assert!(
        tokens
            .iter()
            .any(|token| { token.id() == php_tokenizer::T_ATTRIBUTE && token_text(token) == "#[" }),
        "expected attribute opener token"
    );
    assert!(
        tokens.iter().any(|token| {
            token.id() == php_tokenizer::T_NAME_QUALIFIED && token_text(token) == "Foo\\Bar"
        }),
        "expected qualified namespace token"
    );
    assert!(
        tokens.iter().any(|token| {
            token.id() == php_tokenizer::T_VARIABLE
                && token_text(token) == "$name"
                && token.line() == 2
        }),
        "expected variable token on the second PHP line"
    );
    assert!(
        tokens.iter().any(|token| {
            token.id() == php_tokenizer::T_COMMENT && token_text(token).starts_with("// hi")
        }),
        "expected line comment token"
    );
    assert_eq!(
        php_tokenizer::token_name(php_tokenizer::T_NAME_QUALIFIED),
        "T_NAME_QUALIFIED"
    );
}

#[test]
fn token_get_all_exposes_token_names_text_lengths_and_lines() {
    let execution = run_source(
        r#"<?php
$tokens = token_get_all('<?php
echo $name; // hi
');
foreach ($tokens as $token) {
    if (is_array($token)) {
        $name = token_name($token[0]);
        if ($name == "T_OPEN_TAG" || $name == "T_ECHO" || $name == "T_VARIABLE" || $name == "T_COMMENT") {
            echo $name, ":", strlen($token[1]), ":", $token[2], "\n";
        }
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "T_OPEN_TAG:6:1\nT_ECHO:4:2\nT_VARIABLE:5:2\nT_COMMENT:5:2\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_token_static_tokenize_exposes_objects_names_and_string_conversion() {
    let execution = run_source(
        r#"<?php
$tokens = PhpToken::tokenize("<?php echo 12;");
foreach ($tokens as $token) {
    $name = $token->getTokenName();
    if ($name == "T_OPEN_TAG" || $name == "T_ECHO" || $name == "T_LNUMBER") {
        echo $name, ":", $token->id, ":", strlen($token->text), ":", $token->line, ":", $token->__toString(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "T_OPEN_TAG:389:6:1:<?php \nT_ECHO:291:4:1:echo\nT_LNUMBER:260:2:1:12\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn tokenizer_rejects_nonzero_flags_until_token_parse_boundary_is_supported() {
    let error = runtime_error(r#"<?php token_get_all("<?php echo 1;", 1);"#);

    assert!(
        error
            .message
            .contains("TOKEN_PARSE and non-zero tokenizer flags are not implemented"),
        "{}",
        error.message
    );
}
