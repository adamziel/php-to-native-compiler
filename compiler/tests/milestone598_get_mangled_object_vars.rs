use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn get_mangled_object_vars_includes_non_public_instance_slots_with_mangled_keys() {
    let source = r#"<?php
class Vault {
    public $label;
    protected $code;
    private $pin;
    public static $shared;
}

$vault = new Vault();
$vault->label = "safe";
$vars = get_mangled_object_vars($vault);
$keys = array_keys($vars);

echo count($vars), "\n";
echo strlen($keys[0]), "|", strlen($keys[1]), "|", strlen($keys[2]), "\n";
echo $keys[0] === "label", "|", $keys[1] === "code", "|", $keys[2] === "pin", "\n";
echo $vars[$keys[0]], "|", $vars[$keys[1]] === null, "|", $vars[$keys[2]] === null, "\n";

$call = "get_mangled_object_vars";
$dynamic = $call($vault);
$dynamicKeys = array_keys($dynamic);
echo count($dynamic), "|", strlen($dynamicKeys[1]), "|", strlen($dynamicKeys[2]);
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, "3\n5|7|10\n1||\nsafe|1|1\n3|7|10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_mangled_object_vars_until_native_object_lowering_exists() {
    let error = emit_ir_source(
        "<?php\nclass Vault { private $pin; }\n$vault = new Vault();\necho get_mangled_object_vars($vault);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("object-instantiation")
            || error.message.contains("object metadata builtins"),
        "{}",
        error.message
    );
}
