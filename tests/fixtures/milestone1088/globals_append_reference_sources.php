<?php
$GLOBALS["bag"] = [];
$alias =& $GLOBALS["bag"][];
$alias = "global-alias";
echo $bag[0], "\n";
$GLOBALS["bag"][0] = "global-slot";
echo $alias, "\n";

function bind_nested_global() {
    $nested =& $GLOBALS["bag"]["outer"][];
    $nested = "nested-alias";
    echo $GLOBALS["bag"]["outer"][0], "\n";
    $GLOBALS["bag"]["outer"][0] = "nested-slot";
    echo $nested, "\n";
}

bind_nested_global();
echo $bag["outer"][0];
