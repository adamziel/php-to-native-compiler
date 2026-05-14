<?php
$x = 1 + 2;
$is_three = $x === 3;
$word = $is_three ? "alpha" : "beta";

echo $word !== 1, "\n";
echo $word !== 1.0, "\n";
echo $word !== null, "\n";
echo $word !== true, "\n";
echo "x", $word === 1, "y";
