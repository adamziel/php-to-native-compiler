<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$missing ??= fallback("missing-called", "missing-value");
echo $missing, "\n";

$nullable = null;
$nullable ??= fallback("null-called", "null-value");
echo $nullable, "\n";

$kept = "kept-value";
$kept ??= fallback("kept-called", "replacement");
echo $kept, "\n";

$false = false;
$false ??= fallback("false-called", true);
if ($false === false) {
    echo "false-kept\n";
}

$zero = 0;
$zero ??= fallback("zero-called", 9);
if ($zero === 0) {
    echo "zero-kept\n";
}

$empty = "";
$empty ??= fallback("empty-called", "replacement");
if ($empty === "") {
    echo "empty-string-kept";
}
