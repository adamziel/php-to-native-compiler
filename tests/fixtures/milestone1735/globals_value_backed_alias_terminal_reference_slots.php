<?php
$GLOBALS["globalItems"] = [];
$GLOBALS["globalItems"]["slot"] = "global-seed";
$globalAlias =& $GLOBALS["globalItems"]["slot"];

$globalTarget = [];
$globalTarget["copy"] =& $globalAlias;
$globalTarget["outer"]["copy"] =& $globalAlias;

$globalTarget["copy"] = "global-direct-target-write";
$globalTarget["outer"]["copy"] = "global-nested-target-write";

echo $GLOBALS["globalItems"]["slot"],
    "|",
    $globalAlias,
    "|",
    $globalTarget["copy"],
    "|",
    $globalTarget["outer"]["copy"];
