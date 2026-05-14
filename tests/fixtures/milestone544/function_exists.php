<?php
function local_name() {
    return "ok";
}

echo function_exists("local_name") ? "1" : "0";
echo function_exists("LOCAL_NAME") ? "1" : "0";
echo function_exists("strlen") ? "1" : "0";
echo function_exists("function_exists") ? "1" : "0";
echo function_exists("missing") ? "1" : "0";
echo function_exists("not valid") ? "1" : "0";
echo "\n";
$call = "function_exists";
echo $call("local_name") ? "1" : "0";
