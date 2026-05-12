<?php
$defined = "value";
$nullable = null;
$false = false;
$zero = 0;
$empty = "";
$items = [];
$items["present"] = "array-value";
$items["null"] = null;
$items["false"] = false;
$items["zero"] = 0;
$items["empty"] = "";
$items["2"] = "two";
$key = "present";

echo ($defined ?? $undefined_fallback), "\n";
echo ($missing ?? "missing-fallback"), "\n";
echo ($nullable ?? "null-fallback"), "\n";
if (($false ?? true) === false) {
    echo "false-kept\n";
}
if (($zero ?? 9) === 0) {
    echo "zero-kept\n";
}
if (($empty ?? "fallback") === "") {
    echo "empty-string-kept\n";
}
echo ($items[$key] ?? $undefined_offset_fallback), "\n";
echo ($items["missing"] ?? "missing-key-fallback"), "\n";
echo ($items["null"] ?? "null-key-fallback"), "\n";
if (($items["false"] ?? true) === false) {
    echo "array-false-kept\n";
}
if (($items["zero"] ?? 9) === 0) {
    echo "array-zero-kept\n";
}
if (($items["empty"] ?? "fallback") === "") {
    echo "array-empty-string-kept\n";
}
echo ($items[2] ?? "normalized-missing"), "\n";
echo ($undefined_items["any"] ?? "undefined-array-fallback"), "\n";
$number = 42;
echo ($number["any"] ?? "scalar-target-fallback");
