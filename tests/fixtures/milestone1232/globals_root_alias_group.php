<?php
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$GLOBALS["target"] =& $entry;

echo $target, "|", $GLOBALS["target"], "|", $items["slot"], "|", $other, "\n";

$target = "from-target";
echo $entry, "|", $items["slot"], "|", $GLOBALS["target"], "|", $other, "\n";

$items["slot"] = "from-slot";
echo $target, "|", $GLOBALS["target"], "|", $entry, "|", $other;
