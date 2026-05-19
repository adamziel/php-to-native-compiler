<?php
$GLOBALS["GLOBALS"] = "seed";
$GLOBALS["GLOBALS"] = "via";
echo $GLOBALS["GLOBALS"], "|";

$x = "source";
$GLOBALS["GLOBALS"] =& $x;
$GLOBALS["GLOBALS"] = "bound";
echo $x, "|", $GLOBALS["GLOBALS"];
