<?php
$a = "orig";
$b =& $a;
$snapshot = $GLOBALS;

$snapshot["a"] = "copy";
echo $a, "|", $b, "|", $snapshot["b"], "\n";

$snapshot["b"] = "other";
echo $a, "|", $b, "|", $snapshot["a"];
