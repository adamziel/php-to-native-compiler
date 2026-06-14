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

fn classify_with_pipefail(body: &str) -> String {
    classify_with_options(body, false, true, &[])
}

fn classify_with_harness_programs(body: &str, enabled: bool) -> String {
    classify_with_harness_programs_and_env(body, enabled, &[])
}

fn classify_with_harness_programs_and_env(
    body: &str,
    enabled: bool,
    env: &[(&str, &str)],
) -> String {
    classify_with_options(body, enabled, false, env)
}

fn classify_with_options(
    body: &str,
    enabled: bool,
    pipefail: bool,
    env: &[(&str, &str)],
) -> String {
    let root = temp_dir("ptn-phpt-classifier");
    let phpt = root.join("case.phpt");
    fs::write(&phpt, body).expect("write PHPT");

    let mut command = Command::new("bash");
    for key in [
        "PTN_PHPT_AVAILABLE_LOCALES",
        "PTN_PHPT_PHP_INT_SIZE",
        "SKIP_ASAN",
        "SKIP_MSAN",
        "SKIP_UBSAN",
        "SKIP_PERF_SENSITIVE",
    ] {
        command.env_remove(key);
    }
    if enabled {
        command.env("PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS", "1");
    }
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command
        .arg("-c")
        .arg(if pipefail {
            "set -o pipefail; source tools/phpt-classifier.sh; ptn_phpt_classify_row tests/case.phpt \"$1\""
        } else {
            "source tools/phpt-classifier.sh; ptn_phpt_classify_row tests/case.phpt \"$1\""
        })
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
fn phpt_classifier_file_section_helpers_survive_pipefail() {
    let mut phpt =
        String::from("--TEST--\npipefail early exit\n--FILE--\n<?php\nnew ErrorException();\n");
    for _ in 0..5000 {
        phpt.push_str("echo 1;\n");
    }
    phpt.push_str("--EXPECT--\n");

    let classification = classify_with_pipefail(&phpt);
    assert!(
        classification.starts_with("unsupported-diagnostics-runtime\t"),
        "{classification:?}"
    );
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
fn phpt_classifier_models_static_skipif_preconditions() {
    let sanitizer = "--TEST--\nsanitizer\n--SKIPIF--\n<?php\nif (getenv('SKIP_ASAN')) die('skip asan');\nif (getenv('SKIP_MSAN')) die('skip msan');\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(sanitizer, true);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("sanitizer-env"),
        "{classification:?}"
    );

    let classification =
        classify_with_harness_programs_and_env(sanitizer, true, &[("SKIP_ASAN", "1")]);
    assert!(
        classification.starts_with("skipif-precondition\t") && classification.contains("SKIP_ASAN"),
        "{classification:?}"
    );

    let int64 = "--TEST--\nint64\n--SKIPIF--\n<?php if (PHP_INT_SIZE != 8) die('skip 64-bit only'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(int64, true, &[("PTN_PHPT_PHP_INT_SIZE", "8")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_INT_SIZE"),
        "{classification:?}"
    );

    let int32 = "--TEST--\nint32\n--SKIPIF--\n<?php if (PHP_INT_SIZE != 4) die('skip 32-bit only'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(int32, true, &[("PTN_PHPT_PHP_INT_SIZE", "8")]);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("PHP_INT_SIZE guard"),
        "{classification:?}"
    );

    let locale = "--TEST--\nlocale\n--SKIPIF--\n<?php\nif (!setlocale(LC_ALL, \"de_DE.UTF-8\", \"fr_FR.UTF-8\")) {\n    die('skip locale needed');\n}\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        locale,
        true,
        &[("PTN_PHPT_AVAILABLE_LOCALES", "C:de_DE.utf8")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("locale-availability"),
        "{classification:?}"
    );

    let classification = classify_with_harness_programs_and_env(
        locale,
        true,
        &[("PTN_PHPT_AVAILABLE_LOCALES", "C:POSIX")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("locale availability guard"),
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
            "unsupported-anonymous-class\t",
            "requires anonymous class syntax",
        ),
        (
            "interface implementation",
            "--TEST--\niface\n--FILE--\n<?php\nclass Bag implements ArrayAccess {}\n--EXPECT--\n",
            "unsupported-interface-implementation\t",
            "requires interface implementation checks",
        ),
        (
            "interface declaration",
            "--TEST--\ninterface\n--FILE--\n<?php\ninterface Contract {}\n--EXPECT--\n",
            "unsupported-interface-declaration\t",
            "requires interface declarations",
        ),
        (
            "trait declaration",
            "--TEST--\ntrait\n--FILE--\n<?php\ntrait SharedBehavior {}\n--EXPECT--\n",
            "unsupported-trait-declaration\t",
            "requires trait declarations",
        ),
        (
            "call-site unpack",
            "--TEST--\nunpack\n--FILE--\n<?php\nfunction f(...$args) {}\nf(...[1, 2]);\n--EXPECT--\n",
            "unsupported-call-unpacking\t",
            "requires call-site or array unpacking",
        ),
        (
            "generator yield",
            "--TEST--\nyield\n--FILE--\n<?php\n$fn = fn() => yield 123;\n--EXPECT--\n",
            "unsupported-generator-runtime\t",
            "requires generator/yield lowering",
        ),
        (
            "nullable type hint",
            "--TEST--\nnullable\n--FILE--\n<?php\n$fn = fn(?int... $args): array => $args;\n--EXPECT--\n",
            "unsupported-type-hint\t",
            "requires nullable type-hint metadata",
        ),
        (
            "never return type",
            "--TEST--\nnever\n--FILE--\n<?php\n$fn = fn(): never => 42;\n--EXPECT--\n",
            "unsupported-type-hint\t",
            "requires `never` return type",
        ),
        (
            "static local variable",
            "--TEST--\nstatic local\n--FILE--\n<?php\nfunction next_value() { static $value = 0; return ++$value; }\n--EXPECT--\n",
            "unsupported-function-state\t",
            "requires static local variables",
        ),
        (
            "foreach append read",
            "--TEST--\nappend read\n--FILE--\n<?php\nforeach ($items[] as $value) {}\n--EXPECTF--\n",
            "unsupported-expression-diagnostics\t",
            "requires array-append read diagnostics",
        ),
        (
            "foreach assigns this",
            "--TEST--\nthis target\n--FILE--\n<?php\nforeach ($items as list($this)) {}\n--EXPECTF--\n",
            "unsupported-expression-diagnostics\t",
            "requires foreach assignment diagnostics for `$this`",
        ),
        (
            "variable-variable read",
            "--TEST--\ndynamic read\n--FILE--\n<?php\n$name = 'value';\necho $$name;\n--EXPECT--\n",
            "unsupported-dynamic-symbol\t",
            "requires variable variables",
        ),
        (
            "braced variable-variable write",
            "--TEST--\ndynamic write\n--FILE--\n<?php\n$name = 'value';\n${$name} = 1;\n--EXPECT--\n",
            "unsupported-dynamic-symbol\t",
            "requires variable variables",
        ),
        (
            "variable-variable unset",
            "--TEST--\ndynamic unset\n--FILE--\n<?php\n$name = 'value';\nunset($$name);\n--EXPECT--\n",
            "unsupported-dynamic-symbol\t",
            "requires variable variables",
        ),
        (
            "array internal named argument",
            "--TEST--\nnamed internal\n--FILE--\n<?php\nvar_dump(array_filter([], mode: 1));\n--EXPECT--\n",
            "unsupported-internal-call-binding\t",
            "requires named-argument binding for modeled array internal calls",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
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
fn phpt_classifier_splits_unpacking_blockers() {
    let cases = [
        (
            "call-site unpack",
            "--TEST--\ncall unpack\n--FILE--\n<?php\nfunction collect(...$args) { return $args; }\nvar_dump(collect(...[1, 2]));\n--EXPECT--\n",
        ),
        (
            "array literal unpack",
            "--TEST--\narray unpack\n--FILE--\n<?php\nvar_dump([0, ...[1, 2]]);\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-call-unpacking\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains("requires call-site or array unpacking"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_splits_attribute_metadata_blockers() {
    let cases = [
        (
            "attribute syntax on class",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nclass Bag {}\n--EXPECT--\n",
            "unsupported-attribute-syntax-metadata\t",
            "requires PHP attribute syntax",
        ),
        (
            "attribute syntax on function",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nfunction f() {}\n--EXPECT--\n",
            "unsupported-attribute-syntax-metadata\t",
            "requires PHP attribute syntax",
        ),
        (
            "internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(Attribute::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata",
        ),
        (
            "internal Deprecated attribute object",
            "--TEST--\ndeprecated attribute\n--FILE--\n<?php\n$d = new \\Deprecated(\"message\");\n$d->message = \"updated\";\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
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
fn phpt_classifier_keeps_user_named_arguments_runnable() {
    let classification = classify(
        "--TEST--\nuser named arguments\n--FILE--\n<?php\nfunction pick($left, $right) { return $right; }\necho pick(right: 2, left: 1);\n--EXPECT--\n2\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_array_literals_in_internal_calls_runnable() {
    let classification = classify(
        "--TEST--\narray literal\n--FILE--\n<?php\nvar_dump(array_map(null, [\"name\" => 1]));\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_does_not_treat_static_member_syntax_as_named_internal_argument() {
    let classification = classify(
        "--TEST--\nstatic member in internal call\n--FILE--\n<?php\nclass Bag { public static function values() { return [1]; } }\narray_pop((Bag::values()));\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_allows_first_class_callable_syntax() {
    let cases = [
        (
            "function callable",
            "--TEST--\nfcc function\n--FILE--\n<?php\n$fn = strlen(...);\necho $fn('abc');\n--EXPECT--\n3\n",
        ),
        (
            "static method callable",
            "--TEST--\nfcc static\n--FILE--\n<?php\nclass FccStatic { public static function run($v) { return $v; } }\n$fn = FccStatic::run(...);\necho $fn('ok');\n--EXPECT--\nok\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_plain_heredoc_and_nowdoc_runnable() {
    let cases = [
        (
            "plain heredoc",
            "--TEST--\nheredoc\n--FILE--\n<?php\n$value = <<<TXT\nHello\nTXT;\nvar_dump($value);\n--EXPECT--\nstring(5) \"Hello\"\n",
        ),
        (
            "plain nowdoc",
            "--TEST--\nnowdoc\n--FILE--\n<?php\n$value = <<<'TXT'\n$literal\nTXT;\nvar_dump($value);\n--EXPECT--\nstring(8) \"$literal\"\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_excludes_interpolating_heredoc_bodies() {
    let classification = classify(
        "--TEST--\nheredoc interpolation\n--FILE--\n<?php\n$name = \"world\";\necho <<<TXT\nHello $name\nTXT;\n--EXPECT--\nHello world\n",
    );

    assert!(
        classification.starts_with("unsupported-string-parser\t")
            && classification.contains("requires heredoc interpolation"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_generator_fiber_reference_boundaries() {
    let cases = [
        (
            "fiber by-ref return",
            "--TEST--\nfiber\n--FILE--\n<?php\n$fiber = new Fiber(function &() {\n    Fiber::suspend();\n    return $var;\n});\n--EXPECT--\n",
            "requires Fiber coroutine runtime and by-reference return/getReturn boundary",
        ),
        (
            "non-ref generator iterated by-ref",
            "--TEST--\ngenerator foreach by ref\n--FILE--\n<?php\nfunction gen() { yield; }\n$gen = gen();\nforeach ($gen as &$value) {}\n--EXPECTF--\n",
            "requires generator foreach by-reference iteration boundary",
        ),
        (
            "by-ref generator yielding expression",
            "--TEST--\nyield const by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield \"foo\";\n}\n--EXPECTF--\n",
            "requires by-reference generator yield boundary",
        ),
        (
            "by-ref generator yield from",
            "--TEST--\nyield from by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield from [];\n}\n--EXPECTF--\n",
            "requires generator yield-from delegation diagnostics",
        ),
        (
            "generator foreach cleanup",
            "--TEST--\ngenerator foreach cleanup\n--FILE--\n<?php\nfunction gen(array $array) {\n    foreach ($array as $value) {\n        yield $value;\n    }\n}\n--EXPECT--\n",
            "requires generator suspension cleanup for live foreach variables and premature close",
        ),
        (
            "by-ref function call yielded by ref",
            "--TEST--\nyield ref function call\n--FILE--\n<?php\nfunction &nop(&$var) { return $var; }\nfunction &gen(&$var) {\n    yield nop($var);\n}\n--EXPECT--\n",
            "requires by-reference generator yield boundary",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-generator-runtime\t"),
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
            "unsupported-class-contract-metadata\t",
            "requires abstract class/method contract metadata",
        ),
        (
            "magic method dispatch",
            "--TEST--\nmagic\n--FILE--\n<?php\nclass Bag { public function __get($name) { return 1; } }\n--EXPECT--\n",
            "unsupported-magic-method-metadata\t",
            "requires magic method dispatch/reflection metadata",
        ),
        (
            "object string conversion metadata",
            "--TEST--\nstring conversion\n--FILE--\n<?php\nclass Bag { public function __toString() { return 'bag'; } }\n--EXPECT--\n",
            "unsupported-object-string-conversion-metadata\t",
            "requires object-to-string magic conversion metadata",
        ),
        (
            "autoload",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) {});\n--EXPECT--\n",
            "unsupported-autoload-metadata\t",
            "requires runtime class autoload symbol-table mutation",
        ),
        (
            "reflection closure binding",
            "--TEST--\nreflection\n--FILE--\n<?php\n$r = new ReflectionFunction(fn() => 1);\nvar_dump($r->getClosureThis());\n--EXPECT--\n",
            "unsupported-reflection-metadata\t",
            "requires ReflectionFunction closure binding metadata",
        ),
        (
            "non-public method visibility",
            "--TEST--\nvisibility\n--FILE--\n<?php\nclass Box { private function run() {} }\n--EXPECT--\n",
            "unsupported-method-visibility-metadata\t",
            "requires non-public method visibility dispatch",
        ),
        (
            "non-public property visibility",
            "--TEST--\nproperty visibility\n--FILE--\n<?php\nclass Box { protected $value = 1; }\n--EXPECT--\n",
            "unsupported-property-visibility-metadata\t",
            "requires non-public property visibility metadata",
        ),
        (
            "object vars export",
            "--TEST--\nobject vars\n--FILE--\n<?php\n$object = new stdClass;\nvar_dump(get_object_vars($object));\n--EXPECT--\n",
            "unsupported-object-property-metadata\t",
            "requires get_object_vars() object property-table export",
        ),
        (
            "readonly static property",
            "--TEST--\nreadonly static\n--FILE--\n<?php\nclass Bag { public static readonly int $value; }\n--EXPECT--\n",
            "unsupported-readonly-property-metadata\t",
            "requires readonly static property diagnostics",
        ),
        (
            "readonly constructor promotion",
            "--TEST--\nreadonly promotion\n--FILE--\n<?php\nreadonly class Bag {\n    public function __construct(\n        public int $value\n    ) {}\n}\n--EXPECT--\n",
            "unsupported-property-promotion-metadata\t",
            "requires constructor property promotion metadata",
        ),
        (
            "readonly indirect property mutation",
            "--TEST--\nreadonly indirect mutation\n--FILE--\n<?php\nclass Bag { public readonly array $value; }\n$bag = new Bag();\n$ref =& $bag->value;\n--EXPECT--\n",
            "unsupported-readonly-property-metadata\t",
            "requires indirect readonly property mutation diagnostics",
        ),
        (
            "typed property metadata",
            "--TEST--\ntyped property\n--FILE--\n<?php\nclass Bag { public int $value; }\n--EXPECT--\n",
            "unsupported-typed-property-metadata\t",
            "requires typed property metadata",
        ),
        (
            "typed class constant metadata",
            "--TEST--\ntyped class constant\n--FILE--\n<?php\nclass Bag { const string NAME = 'bag'; }\n--EXPECT--\n",
            "unsupported-class-constant-metadata\t",
            "requires typed class constant metadata",
        ),
        (
            "internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(Attribute::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata",
        ),
        (
            "internal Deprecated attribute object",
            "--TEST--\ndeprecated attribute\n--FILE--\n<?php\n$d = new \\Deprecated(\"message\");\n$d->message = \"updated\";\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata",
        ),
        (
            "complete arginfo registry reflection",
            "--TEST--\narginfo sweep\n--FILE--\n<?php\nforeach (get_defined_functions()[\"internal\"] as $function) { var_dump($function); }\n--EXPECT--\n",
            "unsupported-internal-reflection-metadata\t",
            "requires complete internal arginfo/class registry reflection",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
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
fn phpt_classifier_splits_magic_method_metadata_blockers() {
    let classification = classify(
        "--TEST--\nmagic tostring\n--FILE--\n<?php\nclass Box { public function __toString() { return 'box'; } }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("unsupported-object-string-conversion-metadata\t"),
        "{classification:?}"
    );
    assert!(
        classification.contains("requires object-to-string magic conversion metadata"),
        "{classification:?}"
    );

    let cases = [
        (
            "property magic hook",
            "--TEST--\nmagic get\n--FILE--\n<?php\nclass Box { public function __get($name) { return 1; } }\n--EXPECT--\n",
        ),
        (
            "debug info hook",
            "--TEST--\nmagic debug\n--FILE--\n<?php\nclass Box { public function __debugInfo() { return []; } }\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-magic-method-metadata\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains("requires magic method dispatch/reflection metadata"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_excludes_unsupported_foreach_internal_surfaces() {
    let cases = [
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
fn phpt_classifier_excludes_unsupported_runtime_diagnostics_surfaces() {
    let cases = [
        (
            "backtrace",
            "--TEST--\nbacktrace\n--FILE--\n<?php\nprint_r(debug_backtrace(0, 1));\ndebug_print_backtrace();\n--EXPECT--\n",
            "unsupported-diagnostics-runtime\t",
            "stack-frame snapshots",
        ),
        (
            "user error handler",
            "--TEST--\nhandler\n--FILE--\n<?php\nset_error_handler('handler');\nrestore_error_handler();\n--EXPECT--\n",
            "unsupported-diagnostics-runtime\t",
            "user error/exception handler state",
        ),
        (
            "exception trace metadata",
            "--TEST--\ntrace\n--FILE--\n<?php\ntry { throw new Exception(); } catch (Exception $e) { echo $e->getTraceAsString(); }\n--EXPECT--\n",
            "unsupported-diagnostics-runtime\t",
            "stack-frame snapshots",
        ),
        (
            "error exception metadata",
            "--TEST--\nerror exception\n--FILE--\n<?php\ntry { throw new ErrorException(); } catch (ErrorException $e) { var_dump($e->getSeverity()); }\n--EXPECT--\n",
            "unsupported-diagnostics-runtime\t",
            "ErrorException severity",
        ),
        (
            "assert options",
            "--TEST--\nassert options\n--FILE--\n<?php\nassert_options(ASSERT_BAIL, 1);\nassert(false);\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "assert_options() mode/callback state",
        ),
        (
            "runtime zend assertions",
            "--TEST--\nassert ini\n--FILE--\n<?php\nini_set('zend.assertions', 0);\nvar_dump(assert(false));\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "runtime zend.assertions mode switching",
        ),
        (
            "namespace assert",
            "--TEST--\nnamespace assert\n--FILE--\n<?php\nnamespace Foo;\nvar_dump(assert(false));\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "namespace-aware assertion function resolution",
        ),
        (
            "assert null coalesce assignment",
            "--TEST--\nassert lvalue\n--FILE--\n<?php\nassert($items['key'] ??= 1);\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "assertion expression lvalue mode interaction",
        ),
        (
            "assert closure pretty print",
            "--TEST--\nassert closure\n--FILE--\n<?php\nassert(0 && ($fn = function () { return 1; }));\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "assertion AST pretty-printing for closure expressions",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
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
fn phpt_classifier_keeps_exception_get_trace_runnable() {
    let classification = classify(
        "--TEST--\ntrace\n--FILE--\n<?php\ntry { throw new Exception(); } catch (Exception $e) { var_dump($e->getTrace()); }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_basic_assertions_runnable() {
    let classification = classify(
        "--TEST--\nassert\n--FILE--\n<?php\nvar_dump(assert(true));\ntry { assert(false, 'failed'); } catch (AssertionError $e) { echo $e->getMessage(); }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_basic_assertion_closure_invocation_runnable() {
    let classification = classify(
        "--TEST--\nassert closure invocation\n--FILE--\n<?php\nassert((function () { return true; })());\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
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
            "array_splice destructor reentrancy",
            "--TEST--\nsplice destructor\n--FILE--\n<?php\nclass C { function __destruct() { global $items; $items[] = 0; } }\n$items = [1, new C, 2];\narray_splice($items, 1, 1);\n--EXPECT--\n",
            "requires array_splice() destructor reentrancy detection",
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
fn phpt_classifier_keeps_modeled_mutating_array_helpers_runnable() {
    let cases = [
        (
            "array_splice",
            "--TEST--\nsplice\n--FILE--\n<?php\n$items = [1, 2, 3];\narray_splice($items, 1, 1, [4]);\n--EXPECT--\n",
        ),
        (
            "array_walk_recursive",
            "--TEST--\nrecursive walk\n--FILE--\n<?php\n$items = [1];\narray_walk_recursive($items, \"var_dump\");\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_excludes_huge_array_allocation_rows() {
    let cases = [
        (
            "literal huge count",
            "--TEST--\nhuge array fill\n--FILE--\n<?php\narray_fill(0, 2147483647, 1);\n--EXPECTF--\n",
        ),
        (
            "constant-scale variable count",
            "--TEST--\nhuge array fill variable\n--FILE--\n<?php\n$intMax = PHP_INT_MAX;\narray_fill(0, $intMax, 1);\n--EXPECTF--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-resource-limit\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains("multi-billion element array_fill()"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_large_array_fill_start_key_runnable() {
    let classification = classify(
        "--TEST--\nlarge start key\n--FILE--\n<?php\narray_fill(PHP_INT_MAX, 1, 'x');\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_unsupported_internal_names_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\ninternal names text\n--FILE--\n<?php\n// array_splice($a, 0); debug_backtrace(); get_defined_functions();\n# array_multisort($a)\n/* usort($a, \"cmp\"); array_walk_recursive($a, \"cb\"); ini_set(\"zend.assertions\", 0); */\necho \"array_splice array_multisort usort uasort uksort array_walk_recursive debug_backtrace get_defined_functions ini_set zend.assertions\";\n--EXPECT--\narray_splice array_multisort usort uasort uksort array_walk_recursive debug_backtrace get_defined_functions ini_set zend.assertions\n",
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
fn phpt_classifier_keeps_runtime_diagnostics_words_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\ndiagnostic text\n--FILE--\n<?php\n// debug_backtrace(); set_error_handler('x'); assert_options(ASSERT_BAIL, 1);\n# ini_set('zend.assertions', 0); new ErrorException();\n/* debug_print_backtrace(); restore_error_handler(); */\necho \"debug_backtrace set_error_handler assert_options zend.assertions ErrorException getSeverity\";\n--EXPECT--\ndebug_backtrace set_error_handler assert_options zend.assertions ErrorException getSeverity\n",
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
