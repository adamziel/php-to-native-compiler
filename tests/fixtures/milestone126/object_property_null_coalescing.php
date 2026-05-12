<?php
class Box {
    public $value;
    public $nullable;
    public $flag;
    public $zero;
    public $empty;
}

$box = new Box();
$box->value = "object-value";
$box->flag = false;
$box->zero = 0;
$box->empty = "";

echo ($box->value ?? $undefined_object_fallback), "\n";
echo ($box->nullable ?? "null-property-fallback"), "\n";
echo ($box->missing ?? "missing-property-fallback"), "\n";
if (($box->flag ?? true) === false) {
    echo "object-false-kept\n";
}
if (($box->zero ?? 9) === 0) {
    echo "object-zero-kept\n";
}
if (($box->empty ?? "fallback") === "") {
    echo "object-empty-string-kept\n";
}
echo ($missing_box->value ?? "undefined-object-fallback"), "\n";
$number = 42;
echo ($number->value ?? "non-object-fallback");
