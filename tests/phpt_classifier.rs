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
            "interface implementation",
            "--TEST--\niface\n--FILE--\n<?php\nclass Bag implements ArrayAccess {}\n--EXPECT--\n",
            "requires interface implementation checks",
        ),
        (
            "call-site unpack",
            "--TEST--\nunpack\n--FILE--\n<?php\nfunction f(...$args) {}\nf(...[1, 2]);\n--EXPECT--\n",
            "requires call-site or array unpacking",
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
fn phpt_classifier_keeps_variadic_parameter_rows_runnable() {
    let classification = classify(
        "--TEST--\nvariadic\n--FILE--\n<?php\nfunction f(...$args) { var_dump($args); }\nf(1, 2);\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}
