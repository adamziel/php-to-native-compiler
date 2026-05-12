use php_compiler::run_source;

#[test]
fn multiple_unset_operands_execute_left_to_right() {
    let source = r#"<?php
function pick($label, $key) {
    echo "pick:", $label, "\n";
    return $key;
}

$items = ["name" => "Ada", "city" => "Paris", "role" => "dev"];
$target = "live";

unset($items[pick("first", "name")], $target, $items[pick("second", "city")], $missing);

echo "count:", count($items), "\n";
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
}
if (isset($target)) {
    echo "target:set\n";
} else {
    echo "target:unset\n";
}
if (isset($missing)) {
    echo "missing:set";
} else {
    echo "missing:unset";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "pick:first\npick:second\ncount:1\nrole=dev\ntarget:unset\nmissing:unset"
    );
    assert_eq!(execution.exit_code, 0);
}
