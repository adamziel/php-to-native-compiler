<?php
$value = "first";
$GLOBALS["bag"]["slot"] =& $value;
echo $bag["slot"], "|", $GLOBALS["bag"]["slot"], "|";
$value = "second";
echo $bag["slot"], "|";
$GLOBALS["bag"]["slot"] = "third";
echo $value, "|", $bag["slot"], "\n";

function bind_nested() {
    $local = "local";
    $GLOBALS["from_function"]["slot"] =& $local;
    $local = "changed";
    echo $GLOBALS["from_function"]["slot"], "|";
}

bind_nested();
echo $from_function["slot"], "|";
$from_function["slot"] = "global-write";
echo $GLOBALS["from_function"]["slot"], "\n";

$appended = "append";
$GLOBALS["list"][] =& $appended;
echo $list[0], "|";
$appended = "changed-append";
echo $GLOBALS["list"][0], "|";
$list[0] = "slot-write";
echo $appended;
