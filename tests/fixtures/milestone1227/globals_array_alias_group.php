<?php
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$GLOBALS["bag"]["slot"] =& $entry;

$entry = "from-entry";
echo $items["slot"], "|", $bag["slot"], "|", $other, "\n";

$other = "from-other";
echo $items["slot"], "|", $GLOBALS["bag"]["slot"], "|", $entry, "\n";

$GLOBALS["bag"]["slot"] = "from-global";
echo $entry, "|", $items["slot"], "|", $other;
