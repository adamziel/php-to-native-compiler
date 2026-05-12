<?php
$i = 3;
do {
    echo $i, ":";
    $i = $i + 1;
} while ($i < 3);
echo "\n", "after:", $i, "\n";

$j = 0;
do {
    $j = $j + 1;
    if ($j == 2) {
        continue;
    }
    if ($j == 5) {
        break;
    }
    echo $j, ",";
} while ($j < 10);
echo "\n", "j:", $j, "\n";

DO echo "single"; WHILE (false);
echo "\n";
