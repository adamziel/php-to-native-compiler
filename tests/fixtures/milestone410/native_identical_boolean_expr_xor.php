<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$different = ($is_three xor $is_three);

echo $different ? 1 : 0;
