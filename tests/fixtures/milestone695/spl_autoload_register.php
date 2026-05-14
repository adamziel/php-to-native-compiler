<?php
$called = "no";
echo spl_autoload_register(function ($class) use ($called) {
    echo "called";
    return false;
}) ? "1" : "0";
echo "|", $called, "\n";

$call = "spl_autoload_register";
function MissingAutoloader($class) {
    return false;
}
echo $call("MissingAutoloader", true, false) ? "1" : "0";
