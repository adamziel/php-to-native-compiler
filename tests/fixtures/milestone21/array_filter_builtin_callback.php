<?php
$items = ["empty" => "", "zero" => "0", "space" => " "];
$filtered = array_filter($items, "strlen");
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["zero"], "|", strlen($filtered["space"]);
