<?php
function milestone1902_notice($errno, $errstr) {
    echo "notice:" . (str_contains($errstr, "Only variables") ? "ref" : "other") . "\n";
    return true;
}

set_error_handler("milestone1902_notice", E_NOTICE);

$items = array("x", "y");
$alias =& call_user_func_array("array_pop", array(&$items));
$alias = "z";

$length =& call_user_func("strlen", "abcd");
$length = 9;

echo count($items), "|", implode(",", $items), "|", $alias, "|", $length;
