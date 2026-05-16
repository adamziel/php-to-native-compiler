<?php
function observe_global_value() {
    global $value;
    echo "seen=", $value, "|";
}

function mutate_and_observe(&$param) {
    $param = 2;
    observe_global_value();
    $param = 3;
}

function detach_reference(&$param) {
    unset($param);
    $param = 9;
    echo "local=", $param, "|";
}

$value = 1;
mutate_and_observe($value);
echo "final=", $value, "\n";

detach_reference($value);
echo "caller=", $value;
