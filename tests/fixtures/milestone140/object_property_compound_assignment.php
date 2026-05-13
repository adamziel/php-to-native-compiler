<?php
class Box {
    public $value;
    public $text;
    public $i;
    public $sum;
}

$box = new Box();
$box->value = 10;
$box->text = "php";
$box->value += 5;
$box->value *= "2";
$box->text .= "-native";
echo $box->value, ":", $box->text, "\n";
echo ($box->value -= 4), ":", $box->value, "\n";
echo ($box->value /= 2), ":", $box->value, "\n";

function next_value() {
    echo "rhs\n";
    return 3;
}
echo ($box->value += next_value()), ":", $box->value, "\n";

$box->sum = 0;
for ($box->i = 0; $box->i < 3; $box->i += 1) {
    $box->sum += $box->i;
}
echo $box->sum, ":", $box->i;
