use std::fs;
use std::path::Path;
use std::process::{Command, Output};

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

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}

fn selected_contextual_token_names(tokens: &[PhpTokenizerToken]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|token| {
            let text = token_text(token);
            matches!(text.as_str(), "continue" | "ARRAY" | "namespace")
                .then(|| format!("{}:{text}", php_tokenizer::token_name(token.id())))
        })
        .collect()
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
fn tokenizer_token_parse_contextual_keywords_differ_from_plain_scan() {
    let source = b"<?php X::continue; class C { const ARRAY = 1; use A { namespace as bar; } }";
    let plain = php_tokenizer::tokenize(source);
    let token_parse = php_tokenizer::tokenize_with_token_parse(source, true);

    assert_eq!(
        selected_contextual_token_names(&plain),
        vec![
            "T_CONTINUE:continue",
            "T_ARRAY:ARRAY",
            "T_NAMESPACE:namespace"
        ]
    );
    assert_eq!(
        selected_contextual_token_names(&token_parse),
        vec!["T_STRING:continue", "T_STRING:ARRAY", "T_STRING:namespace"]
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
fn token_get_all_token_parse_preserves_contextual_lexical_rows() {
    let execution = run_source(
        r#"<?php
$code = '<?php
X::continue;
class C { const ARRAY = 1; use A { namespace as bar; } }
';
function dump_selected_tokens($label, $tokens) {
    echo "--", $label, "\n";
    foreach ($tokens as $token) {
        if (is_array($token)) {
            $name = token_name($token[0]);
            if ($token[1] == "continue" || $token[1] == "ARRAY" || $token[1] == "namespace") {
                echo $name, ":", $token[1], "\n";
            }
        }
    }
}
dump_selected_tokens("plain", token_get_all($code));
dump_selected_tokens("parse", token_get_all($code, TOKEN_PARSE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "--plain\n",
            "T_CONTINUE:continue\n",
            "T_ARRAY:ARRAY\n",
            "T_NAMESPACE:namespace\n",
            "--parse\n",
            "T_STRING:continue\n",
            "T_STRING:ARRAY\n",
            "T_STRING:namespace\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn tokenizer_contextual_lexical_fixture_runs_through_phpc_cli() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_arg = "tests/fixtures/milestone2306/tokenizer_token_parse_lexical_context.php";
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["run", fixture_arg])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {fixture_arg}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone2306/tokenizer_token_parse_lexical_context.cli"),
    )
    .unwrap_or_else(|error| panic!("failed to read CLI snapshot for {fixture_arg}: {error}"));
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected, "CLI snapshot mismatch for {fixture_arg}");
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
            .contains("non-zero tokenizer flags other than TOKEN_PARSE"),
        "{}",
        error.message
    );
}
