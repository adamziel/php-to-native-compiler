<?php
$x = "seed";
$GLOBALS["GLOBALS"]["x"] = "via";
$GLOBALS["GLOBALS"]["bag"]["slot"] = "deep";

echo $x, "|", $GLOBALS["x"], "|", $GLOBALS["GLOBALS"]["x"], "|", $GLOBALS["GLOBALS"]["bag"]["slot"];
