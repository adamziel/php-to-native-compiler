<?php
function milestone1899_warning($errno, $errstr) {
    echo "warning:" . (str_contains($errstr, "must be passed by reference") ? "ref" : "other") . "\n";
    return true;
}

set_error_handler("milestone1899_warning", E_WARNING);

$items = array("x", "y");
echo call_user_func("array_pop", $items), "|", count($items), "|", implode(",", $items), "\n";

$letters = array("b");
echo call_user_func("array_unshift", $letters, "a"), "|", implode(",", $letters), "\n";

$cursor = array("first", "second");
echo call_user_func("next", $cursor), "|", current($cursor);
