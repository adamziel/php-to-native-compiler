<?php
$sum = 0;
for ($i = 0; $i < 5; $i = $i + 1) {
    if ($i == 1) {
        continue;
    }
    if ($i == 4) {
        break;
    }
    $sum = $sum + $i;
    echo $i, ":";
}
echo "\n", $sum, "\n";

$j = 0;
for (; $j < 3; $j = $j + 1) {
    echo $j;
}
echo "\n";

$k = 0;
for (; ; $k = $k + 1) {
    if ($k >= 2) {
        break;
    }
    echo "k", $k;
    if ($k < 1) {
        echo "\n";
    }
}
