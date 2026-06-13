use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    dir.push(format!("{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn classify(body: &str) -> String {
    let root = temp_dir("ptn-phpt-classifier");
    let phpt = root.join("case.phpt");
    fs::write(&phpt, body).expect("write PHPT");

    let output = Command::new("bash")
        .arg("-c")
        .arg("source tools/phpt-classifier.sh; ptn_phpt_classify_row tests/case.phpt \"$1\"")
        .arg("bash")
        .arg(&phpt)
        .output()
        .expect("run classifier");
    assert!(
        output.status.success(),
        "classifier failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("classifier output should be utf8")
}

#[test]
fn phpt_classifier_excludes_currently_unsupported_language_surfaces() {
    let cases = [
        (
            "anonymous class",
            "--TEST--\nanon\n--FILE--\n<?php\nvar_dump(new class {});\n--EXPECT--\n",
            "requires anonymous class syntax",
        ),
        (
            "attribute syntax on class",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nclass Bag {}\n--EXPECT--\n",
            "requires PHP attribute syntax",
        ),
        (
            "interface implementation",
            "--TEST--\niface\n--FILE--\n<?php\nclass Bag implements ArrayAccess {}\n--EXPECT--\n",
            "requires interface implementation checks",
        ),
        (
            "attribute syntax",
            "--TEST--\nattr\n--FILE--\n<?php\n#[Deprecated]\nfunction f() {}\n--EXPECT--\n",
            "requires PHP attribute syntax",
        ),
        (
            "call-site unpack",
            "--TEST--\nunpack\n--FILE--\n<?php\nfunction f(...$args) {}\nf(...[1, 2]);\n--EXPECT--\n",
            "requires call-site or array unpacking",
        ),
        (
            "attribute syntax on function",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nfunction f() {}\n--EXPECT--\n",
            "requires PHP attribute syntax",
        ),
        (
            "readonly property modifier",
            "--TEST--\nreadonly property\n--FILE--\n<?php\nclass Bag { public readonly int $value; }\n--EXPECT--\n",
            "requires readonly class/property modifiers",
        ),
        (
            "arrow function",
            "--TEST--\narrow\n--FILE--\n<?php\n$fn = fn($value) => $value + 1;\n--EXPECT--\n",
            "requires arrow function syntax",
        ),
        (
            "userland throw",
            "--TEST--\nthrow\n--FILE--\n<?php\ntry { throw new Exception('boom'); } catch (Exception $e) {}\n--EXPECT--\n",
            "requires userland throw expression/statement lowering",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-language\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_asymmetric_property_visibility_rows_runnable() {
    let classification = classify(
        "--TEST--\nasymmetric visibility\n--FILE--\n<?php\nclass Bag { public private(set) int $value; }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_unsupported_class_metadata_surfaces() {
    let cases = [
        (
            "abstract method contracts",
            "--TEST--\nabstract\n--FILE--\n<?php\nabstract class Base { abstract protected function run(); }\n--EXPECT--\n",
            "requires abstract class/method contract metadata",
        ),
        (
            "autoload",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) {});\n--EXPECT--\n",
            "requires runtime class autoload symbol-table mutation",
        ),
        (
            "non-public method visibility",
            "--TEST--\nvisibility\n--FILE--\n<?php\nclass Box { private function run() {} }\n--EXPECT--\n",
            "requires non-public method visibility dispatch",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-class-metadata\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_variadic_parameter_rows_runnable() {
    let classification = classify(
        "--TEST--\nvariadic\n--FILE--\n<?php\nfunction f(...$args) { var_dump($args); }\nf(1, 2);\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_attribute_text_in_strings_runnable() {
    let classification = classify(
        "--TEST--\nattribute text\n--FILE--\n<?php\necho \"prefix #[not an attribute]\";\n--EXPECT--\nprefix #[not an attribute]\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_unsupported_syntax_words_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\nsyntax text\n--FILE--\n<?php\n// throw new Exception();\n# fn($x) => $x\n/* public private(set) int $value; */\necho \"readonly class fn throw private(set) <<<HEREDOC\";\n--EXPECT--\nreadonly class fn throw private(set) <<<HEREDOC\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_hash_comments_runnable() {
    let classification = classify(
        "--TEST--\ncomment\n--FILE--\n<?php\n# ordinary comment\nvar_dump(1);\n--EXPECT--\nint(1)\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}
