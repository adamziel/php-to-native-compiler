<?php
$letters = array("b");
$args = array(&$letters, "a");
echo call_user_func_array("array_unshift", $args), "|", implode(",", $letters), "|", implode(",", $args[0]), "\n";

$cursor = array("first", "second");
$cursorArgs = array(&$cursor);
echo call_user_func_array("next", $cursorArgs), "|", current($cursor), "|", current($cursorArgs[0]);
