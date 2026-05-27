use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn method_defaults_can_reference_self_class_constants() {
    let execution = run_source(
        r#"<?php
class Defaults {
    const SIZE = 32;
    private const SECRET = "secret";

    public static function stat($length = self::SIZE) {
        echo $length, "\n";
    }

    public function inst($label = self::SECRET, $suffix = ":" . self::SIZE) {
        echo $label, $suffix, "\n";
    }
}

class BaseDefaults {
    const SIZE = 16;

    public static function inherited($length = self::SIZE) {
        echo $length, "\n";
    }
}

class ChildDefaults extends BaseDefaults {
    const SIZE = 64;
}

Defaults::stat();
Defaults::stat(64);
$defaults = new Defaults();
$defaults->inst();
$defaults->inst("manual", ":8");
ChildDefaults::inherited();
ChildDefaults::inherited(128);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "32\n64\nsecret:32\nmanual:8\n16\n128\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn self_class_constant_defaults_need_class_context_when_omitted() {
    let error = run_source(
        r#"<?php
function broken($value = self::MISSING) {
    echo $value;
}

broken();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 30);
    assert_eq!(
        error.message,
        "unsupported call self::MISSING: self class constant access requires instance method context"
    );
}

#[test]
fn broader_class_constant_defaults_remain_explicitly_unsupported() {
    let error = run_source(
        r#"<?php
class Defaults {
    const SIZE = 32;
}

function broken($value = Defaults::SIZE) {
    echo $value;
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 6);
    assert_eq!(error.column, 34);
    assert_eq!(
        error.message,
        "default parameter values only support constant expressions in the current subset"
    );
}

#[test]
fn emit_ir_rejects_self_class_constant_defaults_with_class_declarations() {
    let error = emit_ir_source(
        r#"<?php
class Defaults {
    const SIZE = 32;
    public static function stat($length = self::SIZE) {
        echo $length;
    }
}
Defaults::stat();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "{}",
        error.message
    );
}
