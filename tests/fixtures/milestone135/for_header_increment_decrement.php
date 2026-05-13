<?php
$sum = 0;
for ($i = 0; $i < 4; $i++) {
    $sum += $i;
}
echo $sum, "\n";

for (++$sum; $sum > 3; --$sum) {
    echo $sum, "\n";
}
echo "done:", $sum, "\n";
