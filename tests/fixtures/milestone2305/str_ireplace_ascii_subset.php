<?php
$count = 0;
echo str_ireplace("tt", "a", "ttttTttttttttTT", $count), "|", $count, "\n";
$count = 0;
echo str_ireplace(array("%0d", "%0a"), "", "%0%0DDD%0a", $count), "|", $count, "\n";
$call = "str_ireplace";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call("wp", "php", "WP wp", $count), "|", $count, "\n";
echo call_user_func("str_ireplace", "A", "x", "aA"), "\n";
$count = 0;
echo call_user_func_array("str_ireplace", array("a", "b", "BanAna", "count" => &$count)), "|", $count;
