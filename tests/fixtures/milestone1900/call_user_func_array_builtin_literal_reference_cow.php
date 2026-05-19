<?php
$items = array("x", "y");
echo call_user_func_array("array_pop", array(&$items)), "|", count($items), "|", implode(",", $items), "\n";

$keys = array(10 => "ten", 2 => "two");
echo call_user_func_array("ksort", array(&$keys, SORT_NUMERIC)), "|", implode(",", array_keys($keys));
