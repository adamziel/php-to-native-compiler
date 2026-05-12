<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$items = [];
$items["missing"] ??= fallback("missing-called", "missing-value");
echo $items["missing"], "\n";

$items["null"] = null;
$items["null"] ??= fallback("null-called", "null-value");
echo $items["null"], "\n";

$items["kept"] = "kept-value";
$items["kept"] ??= fallback("kept-called", "replacement");
echo $items["kept"], "\n";

$items["false"] = false;
$items["false"] ??= fallback("false-called", true);
if ($items["false"] === false) {
    echo "false-kept\n";
}

$items["zero"] = 0;
$items["zero"] ??= fallback("zero-called", 9);
if ($items["zero"] === 0) {
    echo "zero-kept\n";
}

$items["empty"] = "";
$items["empty"] ??= fallback("empty-called", "replacement");
if ($items["empty"] === "") {
    echo "empty-string-kept\n";
}

$undefined_items["created"] ??= fallback("undefined-array-called", "created-value");
echo $undefined_items["created"], "\n";

$nullable_items = null;
$nullable_items["created"] ??= fallback("null-array-called", "null-created-value");
echo $nullable_items["created"], "\n";

$numeric_keys["2"] ??= fallback("numeric-key-called", "two");
echo $numeric_keys[2];
