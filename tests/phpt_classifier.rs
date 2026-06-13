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
    classify_with_harness_programs(body, false)
}

fn classify_with_harness_programs(body: &str, enabled: bool) -> String {
    let root = temp_dir("ptn-phpt-classifier");
    let phpt = root.join("case.phpt");
    fs::write(&phpt, body).expect("write PHPT");

    let mut command = Command::new("bash");
    if enabled {
        command.env("PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS", "1");
    }
    let output = command
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
fn phpt_classifier_skipif_harness_is_opt_in() {
    let skipif = "--TEST--\nskipif\n--SKIPIF--\n<?php echo getenv('PTN_SKIP') ? 'skip' : ''; ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";

    assert!(classify(skipif).starts_with("runnable\t"));

    let classification = classify_with_harness_programs(skipif, true);
    assert!(
        classification.starts_with("harness-skipif\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_cleanup_harness_is_default_blocker() {
    let cleanup = "--TEST--\ncleanup\n--FILE--\n<?php echo 1; ?>\n--CLEAN--\n<?php unlink(__DIR__ . '/case.tmp'); ?>\n--EXPECT--\n1\n";
    let classification = classify(cleanup);

    assert!(
        classification.starts_with("harness-cleanup\t"),
        "{classification:?}"
    );
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
            "generator yield",
            "--TEST--\nyield\n--FILE--\n<?php\n$fn = fn() => yield 123;\n--EXPECT--\n",
            "requires generator/yield lowering",
        ),
        (
            "nullable type hint",
            "--TEST--\nnullable\n--FILE--\n<?php\n$fn = fn(?int... $args): array => $args;\n--EXPECT--\n",
            "requires nullable type-hint metadata",
        ),
        (
            "never return type",
            "--TEST--\nnever\n--FILE--\n<?php\n$fn = fn(): never => 42;\n--EXPECT--\n",
            "requires `never` return type",
        ),
        (
            "static local variable",
            "--TEST--\nstatic local\n--FILE--\n<?php\nfunction next_value() { static $value = 0; return ++$value; }\n--EXPECT--\n",
            "requires static local variables",
        ),
        (
            "foreach append read",
            "--TEST--\nappend read\n--FILE--\n<?php\nforeach ($items[] as $value) {}\n--EXPECTF--\n",
            "requires array-append read diagnostics",
        ),
        (
            "foreach assigns this",
            "--TEST--\nthis target\n--FILE--\n<?php\nforeach ($items as list($this)) {}\n--EXPECTF--\n",
            "requires foreach assignment diagnostics for `$this`",
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
fn phpt_classifier_keeps_readonly_property_rows_runnable() {
    let cases = [
        "--TEST--\nreadonly property\n--FILE--\n<?php\nclass Bag { public readonly int $value; }\n--EXPECT--\n",
        "--TEST--\nreadonly class\n--FILE--\n<?php\nreadonly class Bag { public int $value; }\n--EXPECT--\n",
    ];

    for phpt in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_arrow_functions_runnable() {
    let classification = classify(
        "--TEST--\narrow\n--FILE--\n<?php\n$fn = fn($value) => $value + 1;\nvar_dump($fn(1));\n--EXPECT--\nint(2)\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
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
            "reflection closure binding",
            "--TEST--\nreflection\n--FILE--\n<?php\n$r = new ReflectionFunction(fn() => 1);\nvar_dump($r->getClosureThis());\n--EXPECT--\n",
            "requires ReflectionFunction closure binding metadata",
        ),
        (
            "non-public method visibility",
            "--TEST--\nvisibility\n--FILE--\n<?php\nclass Box { private function run() {} }\n--EXPECT--\n",
            "requires non-public method visibility dispatch",
        ),
        (
            "non-public property visibility",
            "--TEST--\nproperty visibility\n--FILE--\n<?php\nclass Box { protected $value = 1; }\n--EXPECT--\n",
            "requires non-public property visibility metadata",
        ),
        (
            "object vars export",
            "--TEST--\nobject vars\n--FILE--\n<?php\n$object = new stdClass;\nvar_dump(get_object_vars($object));\n--EXPECT--\n",
            "requires get_object_vars() object property-table export",
        ),
        (
            "readonly static property",
            "--TEST--\nreadonly static\n--FILE--\n<?php\nclass Bag { public static readonly int $value; }\n--EXPECT--\n",
            "requires readonly static property diagnostics",
        ),
        (
            "readonly constructor promotion",
            "--TEST--\nreadonly promotion\n--FILE--\n<?php\nreadonly class Bag {\n    public function __construct(\n        public int $value\n    ) {}\n}\n--EXPECT--\n",
            "requires constructor property promotion metadata",
        ),
        (
            "readonly indirect property mutation",
            "--TEST--\nreadonly indirect mutation\n--FILE--\n<?php\nclass Bag { public readonly array $value; }\n$bag = new Bag();\n$ref =& $bag->value;\n--EXPECT--\n",
            "requires indirect readonly property mutation diagnostics",
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
fn phpt_classifier_excludes_unsupported_foreach_internal_surfaces() {
    let cases = [
        (
            "array_walk_recursive",
            "--TEST--\nrecursive walk\n--FILE--\n<?php\narray_walk_recursive($items, 'visit');\n--EXPECT--\n",
            "requires array_walk_recursive() recursive by-reference callback traversal",
        ),
        (
            "by-ref foreach positional mutation",
            "--TEST--\nforeach mutation\n--FILE--\n<?php\nforeach ($items as &$item) { array_unshift($items, 0); }\n--EXPECT--\n",
            "requires by-reference foreach iterator-pointer preservation",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-internal\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_splits_unsupported_ini_blockers_by_runtime_surface() {
    let cases = [
        (
            "assertion mode",
            "assert.exception=1",
            "unsupported-assertion-ini\t",
            "configurable assert.exception assertion mode",
        ),
        (
            "request input",
            "enable_post_data_reading=0",
            "unsupported-request-input-ini\t",
            "request/input/upload SAPI state",
        ),
        (
            "resource limits",
            "memory_limit=2M",
            "unsupported-resource-limit-ini\t",
            "memory_limit parsing/enforcement",
        ),
        (
            "diagnostics",
            "fatal_error_backtraces=0",
            "unsupported-diagnostics-ini\t",
            "engine diagnostic/logging mode",
        ),
        (
            "function disabling",
            "disable_functions=assert",
            "unsupported-function-disable-ini\t",
            "runtime function table mutation",
        ),
        (
            "opcache",
            "opcache.enable_cli=1",
            "unsupported-opcache-ini\t",
            "Zend OPcache configuration",
        ),
        (
            "scalar formatting",
            "serialize_precision=17",
            "unsupported-scalar-format-ini\t",
            "runtime scalar/string formatting default",
        ),
        (
            "host path",
            "sys_temp_dir=/tmp",
            "unsupported-host-path-ini\t",
            "host path ini",
        ),
    ];

    for (name, ini, category, reason) in cases {
        let classification = classify(&format!(
            "--TEST--\n{name}\n--INI--\n{ini}\n--FILE--\n<?php\necho \"ok\\n\";\n--EXPECT--\nok\n"
        ));
        assert!(
            classification.starts_with(category),
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
fn phpt_classifier_excludes_unsupported_mutating_array_internals() {
    let cases = [
        (
            "array_splice",
            "--TEST--\nsplice\n--FILE--\n<?php\n$items = [1, 2, 3];\narray_splice($items, 1, 1, [4]);\n--EXPECT--\n",
            "requires array_splice() by-reference array mutation",
        ),
        (
            "array_walk_recursive",
            "--TEST--\nrecursive walk\n--FILE--\n<?php\narray_walk_recursive([1], \"var_dump\");\n--EXPECT--\n",
            "requires array_walk_recursive() recursive by-reference callback traversal",
        ),
        (
            "array_multisort",
            "--TEST--\nmultisort\n--FILE--\n<?php\n$left = [2, 1];\n$right = [\"b\", \"a\"];\narray_multisort($left, $right);\n--EXPECT--\n",
            "requires array_multisort() multi-array by-reference sorting",
        ),
        (
            "user comparator sort",
            "--TEST--\nusort\n--FILE--\n<?php\n$items = [3, 1, 2];\nusort($items, \"strcmp\");\n--EXPECT--\n",
            "requires usort()/uasort()/uksort() user-comparator by-reference sort helpers",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-internal\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_unsupported_internal_names_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\ninternal names text\n--FILE--\n<?php\n// array_splice($a, 0);\n# array_multisort($a)\n/* usort($a, \"cmp\"); array_walk_recursive($a, \"cb\"); */\necho \"array_splice array_multisort usort uasort uksort array_walk_recursive\";\n--EXPECT--\narray_splice array_multisort usort uasort uksort array_walk_recursive\n",
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
        "--TEST--\nsyntax text\n--FILE--\n<?php\n// throw new Exception();\n# fn($x) => $x\n/* public private(set) int $value; static $value; array_walk_recursive($a, 'f'); */\necho \"readonly class fn throw private(set) static $value array_walk_recursive($a, 'f') <<<HEREDOC\";\n--EXPECT--\nreadonly class fn throw private(set) static $value array_walk_recursive($a, 'f') <<<HEREDOC\n",
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
