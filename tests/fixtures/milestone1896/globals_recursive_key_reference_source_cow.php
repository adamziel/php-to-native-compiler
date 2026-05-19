<?php
$x = array("slot" => "seed");
$alias =& $GLOBALS["GLOBALS"]["x"]["slot"];
$alias = "via";

echo $x["slot"], "|", $GLOBALS["x"]["slot"], "|", $GLOBALS["GLOBALS"]["x"]["slot"], "|";

$GLOBALS["GLOBALS"]["x"]["slot"] = "slot";
echo $alias;
