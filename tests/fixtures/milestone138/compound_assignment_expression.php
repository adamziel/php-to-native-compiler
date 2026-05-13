<?php
$value = 10;
echo ($value += 5), ":", $value, "\n";
echo (($value *= 2) + 1), ":", $value, "\n";
$value /= 3;
echo ($value -= 4), ":", $value, "\n";

$text = "php";
echo ($text .= "-native"), ":", $text, "\n";

function next_value() {
    echo "rhs\n";
    return 2;
}
echo ($value += next_value()), ":", $value, "\n";

