<?php
$formatted = number_format(-2000, 2768);
echo strlen($formatted), "\n";
echo substr($formatted, 0, 7), "\n";
echo substr($formatted, -8), "\n";
echo ($formatted === "-2,000." . str_repeat("0", 2768)) ? "match" : "mismatch";
