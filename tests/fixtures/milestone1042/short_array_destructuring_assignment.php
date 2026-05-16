<?php
[$a, $b] = ["zero", "one", "ignored"];
echo $a, "|", $b, "\n";

$items = ["name" => "Ada", 1 => "one", "0" => "zero"];
[$first, $second] = $items;
echo $first, "|", $second, "\n";

[, $textdomain, $language] = ["full-match", "default", "en_US"];
echo $textdomain, "|", $language, "\n";

[$left, , $right,] = ["left", "skip", "right"];
echo $left, "|", $right;
