<?php
$checks = 0;
for ($i = 0, $j = 10; $checks = $checks + 1, $i < 3; $i = $i + 1, $j = $j + 10) {
    echo $i, ":", $j, "\n";
    if ($i == 1) {
        continue;
    }
}
echo "checks:", $checks;
