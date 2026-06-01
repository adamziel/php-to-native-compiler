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
fn tokenizer_accepts_token_parse_and_contextual_reserved_member_names() {
    let execution = run_source(
        r#"<?php
$tokens = token_get_all('<?php
X::continue;
$x->class;
class X {
    const ARRAY = 1;
    public $x = self::ARRAY;
}
', TOKEN_PARSE);
foreach ($tokens as $token) {
    if (is_array($token)) {
        $name = token_name($token[0]);
        if ($name == "T_WHITESPACE" || $name == "T_OPEN_TAG") {
            continue;
        }
        echo $name, ":", $token[1], "\n";
    } else if ($token == ";" || $token == "{" || $token == "}") {
        echo $token, "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "T_STRING:X\n",
            "T_DOUBLE_COLON:::\n",
            "T_STRING:continue\n",
            ";\n",
            "T_VARIABLE:$x\n",
            "T_OBJECT_OPERATOR:->\n",
            "T_STRING:class\n",
            ";\n",
            "T_CLASS:class\n",
            "T_STRING:X\n",
            "{\n",
            "T_CONST:const\n",
            "T_STRING:ARRAY\n",
            "T_LNUMBER:1\n",
            ";\n",
            "T_PUBLIC:public\n",
            "T_VARIABLE:$x\n",
            "T_STRING:self\n",
            "T_DOUBLE_COLON:::\n",
            "T_STRING:ARRAY\n",
            ";\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn tokenizer_parse_mode_keeps_trait_alias_namespace_as_string() {
    let execution = run_source(
        r#"<?php
$tokens = PhpToken::tokenize('<?php
class C {
    use A {
        namespace as bar;
    }
}
', TOKEN_PARSE);
foreach ($tokens as $token) {
    if ($token->text == "namespace") {
        echo $token->getTokenName(), ":", $token->text, "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "T_STRING:namespace\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn tokenizer_scans_invalid_octal_boundaries_and_underscores() {
    let tokens = php_tokenizer::tokenize(b"<?php 0_10000000000000000000009;");
    let number = tokens
        .iter()
        .find(|token| token_text(token) == "0_10000000000000000000009")
        .expect("underscored invalid octal is a single numeric token");
    assert_eq!(number.id(), php_tokenizer::T_LNUMBER);

    let overflow = php_tokenizer::tokenize(b"<?php 0177777777777777777777787");
    let number = overflow
        .iter()
        .find(|token| token_text(token) == "0177777777777777777777787")
        .expect("overflowing invalid octal is a single numeric token");
    assert_eq!(number.id(), php_tokenizer::T_DNUMBER);

    let invalid_separator = php_tokenizer::tokenize(b"<?php 1__2;");
    assert!(invalid_separator
        .iter()
        .any(|token| token.id() == php_tokenizer::T_LNUMBER && token_text(token) == "1"));
    assert!(invalid_separator
        .iter()
        .any(|token| token.id() == php_tokenizer::T_STRING && token_text(token) == "__2"));
}

#[test]
fn tokenizer_token_parse_deprecated_cast_calls_error_handler() {
    let execution = run_source(
        r#"<?php
set_error_handler(function (int $errno, string $msg) {
    echo "handler:", $msg, "\n";
});
$tokens = PhpToken::tokenize('<?php echo "(double)"; (double) $x;', TOKEN_PARSE);
foreach ($tokens as $token) {
    if (!$token->isIgnorable()) {
        echo $token->getTokenName(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "handler:Non-canonical cast (double) is deprecated, use the (float) cast instead\n",
            "T_ECHO\n",
            "T_CONSTANT_ENCAPSED_STRING\n",
            ";\n",
            "T_DOUBLE_CAST\n",
            "T_VARIABLE\n",
            ";\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_token_objects_support_constructor_methods_subclasses_and_ampersands() {
    let execution = run_source(
        r#"<?php
$token = new PhpToken(T_FUNCTION, "function");
echo $token->getTokenName(), ":", $token->line, ":", $token->pos, "\n";
var_dump($token->is(T_FUNCTION));
var_dump($token->is("function"));
var_dump($token->is([T_CLASS, "function"]));
var_dump($token->isIgnorable());
var_dump((new PhpToken(100000, "x"))->getTokenName());
echo (new PhpToken(40, "("))->getTokenName(), "\n";

$tokens = PhpToken::tokenize('<?php $x & $y;');
foreach ($tokens as $part) {
    if ($part->getTokenName() == "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG") {
        echo $part->getTokenName(), "\n";
    }
}

class MyPhpToken extends PhpToken {
    public int $extra = 123;
    public function lowered(): string {
        return strtolower($this->text);
    }
}
$sub = MyPhpToken::tokenize('<?PHP ECHO "X";');
var_dump($sub[0] instanceof MyPhpToken);
echo $sub[0]->extra, ":", $sub[1]->lowered(), "\n";

unset($token->id);
try {
    $token->is(T_FUNCTION);
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "T_FUNCTION:-1:-1\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "NULL\n",
            "(\n",
            "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG\n",
            "bool(true)\n",
            "123:echo\n",
            "Typed property PhpToken::$id must not be accessed before initialization\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_token_constructor_is_final_for_subclasses() {
    let execution = run_source(
        r#"<?php
class BadPhpToken extends PhpToken {
    public function __construct() {}
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "");
    assert_eq!(
        execution.stderr,
        "Fatal error: Cannot override final method PhpToken::__construct() in Command line code on line 3"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn deferred_instance_property_default_errors_use_allocation_call_sites() {
    let declaration_only = run_source(
        r#"<?php
class A { public $extra = UNKNOWN; }
echo "declared";
"#,
    )
    .unwrap();
    assert_eq!(declaration_only.stdout, "declared");
    assert_eq!(declaration_only.exit_code, 0);

    let ordinary_new = run_source(
        r#"<?php
class A { public $extra = UNKNOWN; }
try {
    new A();
} catch (Error $e) {
    echo $e->getMessage(), ":", $e->getLine(), "\n";
}
"#,
    )
    .unwrap();
    assert_eq!(ordinary_new.stdout, "Undefined constant \"UNKNOWN\":4\n");
    assert_eq!(ordinary_new.exit_code, 0);

    let tokenized_subclass = run_source(
        r#"<?php
class MyPhpToken1 extends PhpToken {
    public $extra = UNKNOWN;
}
try {
    MyPhpToken1::tokenize("<?php foo");
} catch (Error $e) {
    echo $e->getMessage(), ":", $e->getLine(), "\n";
}
"#,
    )
    .unwrap();
    assert_eq!(
        tokenized_subclass.stdout,
        "Undefined constant \"UNKNOWN\":6\n"
    );
    assert_eq!(tokenized_subclass.exit_code, 0);
}

#[test]
fn tokenizer_rejects_unsupported_nonzero_flags() {
    let error = runtime_error(r#"<?php token_get_all("<?php echo 1;", 2);"#);

    assert!(
        error
            .message
            .contains("unsupported non-zero tokenizer flags are not implemented"),
        "{}",
        error.message
    );
}
