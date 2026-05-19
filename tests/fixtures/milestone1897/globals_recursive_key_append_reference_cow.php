<?php
function milestone1897_bind_recursive_globals_append() {
    $alias =& $GLOBALS["GLOBALS"]["bag"][];
    $alias = "via";
    echo $GLOBALS["GLOBALS"]["bag"][0], "|";

    $GLOBALS["GLOBALS"]["bag"][0] = "slot";
    echo $alias;
}

milestone1897_bind_recursive_globals_append();
echo "|", $GLOBALS["GLOBALS"]["bag"][0];
