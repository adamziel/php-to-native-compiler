<?php
function local_name() {
    return "ok";
}

echo is_callable("local_name") ? "1" : "0";
echo is_callable("LOCAL_NAME") ? "1" : "0";
echo is_callable("strlen") ? "1" : "0";
echo is_callable("missing") ? "1" : "0";
echo is_callable(42) ? "1" : "0";
echo "\n";
$call = "is_callable";
echo $call("local_name") ? "1" : "0";
