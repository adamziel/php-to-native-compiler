<?php
$int = 1 + 2;
$float = 1.25 + 2.5;

echo (1 && $int) ? 1 : 0, "\n";
echo (0 || $float) ? 1 : 0, "\n";
echo (0.0 || "0") ? 1 : 0, "\n";
echo ("php" xor 0) ? 1 : 0, "\n";
echo ($int && $float) ? 1 : 0, "\n";
echo ("" xor 0.0) ? 1 : 0;
