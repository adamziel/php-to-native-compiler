<?php
$counter = 0;
$option = "autoload";
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};

echo call_user_func_array($callback, array(&$option, "closure")), "|", $option, "|", $counter, "\n";
echo call_user_func_array($callback, array(&$option, "again")), "|", $option, "|", $counter;
