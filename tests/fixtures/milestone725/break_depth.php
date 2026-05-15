<?php
$i = 0;
while ($i < 3) {
    $j = 0;
    while ($j < 3) {
        echo $i, ":", $j, "\n";
        break 2;
    }
    echo "inner-after\n";
    $i = $i + 1;
}
echo "done\n";
