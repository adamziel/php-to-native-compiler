<?php
class Box {
    public $value;
    public $nullable;
    public $kept;
    public $flag;
    public $zero;
    public $empty;
}

function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$box = new Box();
$box->value ??= fallback("value-called", "object-value");
echo $box->value, "\n";

$box->nullable = null;
$box->nullable ??= fallback("null-called", "null-value");
echo $box->nullable, "\n";

$box->kept = "kept-value";
$box->kept ??= fallback("kept-called", "replacement");
echo $box->kept, "\n";

$box->flag = false;
$box->flag ??= fallback("false-called", true);
if ($box->flag === false) {
    echo "false-kept\n";
}

$box->zero = 0;
$box->zero ??= fallback("zero-called", 9);
if ($box->zero === 0) {
    echo "zero-kept\n";
}

$box->empty = "";
$box->empty ??= fallback("empty-called", "replacement");
if ($box->empty === "") {
    echo "empty-string-kept";
}
