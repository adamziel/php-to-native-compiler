<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

echo ($missing ??= fallback("missing-called", "missing-value")), ":", $missing, "\n";

$nullable = null;
echo ($nullable ??= fallback("null-called", "null-value")), ":", $nullable, "\n";

$kept = "kept-value";
echo ($kept ??= fallback("kept-called", "replacement")), ":", $kept, "\n";

$items = [];
echo ($items["missing"] ??= fallback("array-missing-called", "array-missing")), ":", $items["missing"], "\n";
$items["kept"] = "array-kept";
echo ($items["kept"] ??= fallback("array-kept-called", "replacement")), ":", $items["kept"], "\n";

class Box {
    public $value;
    public $kept;
}

$box = new Box();
echo ($box->value ??= fallback("object-null-called", "object-value")), ":", $box->value, "\n";
$box->kept = "object-kept";
echo ($box->kept ??= fallback("object-kept-called", "replacement")), ":", $box->kept;
