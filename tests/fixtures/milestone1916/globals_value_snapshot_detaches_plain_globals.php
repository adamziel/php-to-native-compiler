<?php
$x = "orig";
$snapshot = $GLOBALS;
$x = "changed";
$snapshot["x"] = "copy";

echo $x, "|", $GLOBALS["x"], "|", $snapshot["x"];
