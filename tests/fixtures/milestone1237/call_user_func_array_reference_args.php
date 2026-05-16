<?php
function update_option_like(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$callback = "update_option_like";
echo call_user_func_array($callback, array(&$option, "cache")), "\n";
echo $option;
