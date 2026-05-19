<?php
$x = "global";

function milestone1919_read_globals() {
    $x = "local";
    $snapshot = $GLOBALS;
    $snapshot["x"] = "copy";
    echo $x, "|", $GLOBALS["x"], "|", $snapshot["x"];
}

milestone1919_read_globals();
echo "|", $x;
