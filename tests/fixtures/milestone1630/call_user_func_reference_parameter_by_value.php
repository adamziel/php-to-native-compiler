<?php
function milestone1630_warning($errno, $errstr) {
    echo "warning:" . $errno . ":" . (str_contains($errstr, "must be passed by reference") ? "ref" : "other") . "\n";
    return true;
}

function milestone1630_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

set_error_handler("milestone1630_warning", E_WARNING);
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
echo call_user_func("milestone1630_mark", $option, "direct"), "|", $option, "\n";
echo call_user_func("milestone1630_mark", $items["payload"]["slot"], "slot"), "|", $items["payload"]["slot"], "\n";
$counter = 0;
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};
echo call_user_func($callback, $option, "closure"), "|", $option, "|", $counter;
