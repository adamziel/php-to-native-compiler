<?php
$value = "first";
$GLOBALS["target"] =& $value;
echo $target, "|", $GLOBALS["target"], "|";
$value = "second";
echo $target, "|", $GLOBALS["target"], "|";
$GLOBALS["target"] = "third";
echo $value, "|", $target, "\n";

function bind_target() {
    $local = "local";
    $GLOBALS["from_function"] =& $local;
    $local = "changed";
    echo $GLOBALS["from_function"], "|";
}

bind_target();
echo $from_function, "|";
$from_function = "global-write";
echo $GLOBALS["from_function"], "\n";

$detached = "kept";
$GLOBALS["detached_target"] =& $detached;
unset($detached);
$detached = "new";
echo $GLOBALS["detached_target"], "|", $detached;
