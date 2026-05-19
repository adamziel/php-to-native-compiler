<?php
$leaf = "orig";
$bag = array("slot" => &$leaf, "plain" => "plain");
$snapshot = $GLOBALS;

$snapshot["bag"]["slot"] = "copy";
$snapshot["bag"]["plain"] = "snapshot";
echo $leaf, "|", $bag["slot"], "|", $snapshot["bag"]["slot"], "|", $bag["plain"], "|", $snapshot["bag"]["plain"], "\n";

$leaf = "new";
echo $snapshot["bag"]["slot"], "|", $bag["slot"];
