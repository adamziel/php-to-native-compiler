use php_compiler::run_source;

#[test]
fn foreach_supports_positional_value_destructuring() {
    let execution = run_source(
        r#"<?php
$rows = [["alpha", 10, "skip"], ["beta", 20, "skip"]];
$out = [];
foreach ($rows as [$name, $score,]) {
    $out[] = $name . ":" . $score;
}
foreach ($rows as $index => list(, $score)) {
    $out[] = $index . "=" . $score;
}
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "alpha:10|beta:20|0=10|1=20");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_rejects_by_reference_destructuring() {
    let error = run_source("<?php\nforeach ([[1]] as [&$value]) {}\n").unwrap_err();

    assert!(
        error
            .message
            .contains("by-value positional destructuring loop targets"),
        "{}",
        error.message
    );
}
