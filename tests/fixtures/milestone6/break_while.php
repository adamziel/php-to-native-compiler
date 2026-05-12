<?php
$i = 0;
while ($i < 5) {
    echo $i, ",";
    if ($i == 2) {
        break;
    }
    $i = $i + 1;
}
echo "after:", $i;
