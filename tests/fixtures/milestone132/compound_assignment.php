<?php
$value = 10;
$value += 5;
echo $value, "\n";
$value -= 3;
echo $value, "\n";
$value *= "2";
echo $value, "\n";
$value /= 4;
echo $value, "\n";

$text = "php";
$text .= "-native";
echo $text, "\n";

$sum = 0;
for ($i = 0; $i < 5; $i += 2) {
    $sum += $i;
}
echo $sum, "\n";
