<?php
$flag = true;
if (false) {
    echo "bad\n";
} elseif ($flag) {
    echo "elseif\n";
} else {
    echo "else\n";
}

$i = 0;
while ($i < 4) {
    $i = $i + 1;
    if ($i == 2) {
        continue;
    }
    if ($i == 4) {
        break;
    }
    echo "w", $i, "\n";
}

for ($j = 0; $j < 3; $j = $j + 1) {
    echo "f", $j, "\n";
}

$k = 0;
do {
    echo "d", $k, "\n";
    $k = $k + 1;
} while ($k < 2);

switch ($i) {
    case 4:
        echo "switch";
        break;
    default:
        echo "default\n";
}
