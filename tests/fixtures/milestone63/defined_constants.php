<?php
echo defined("ARRAY_FILTER_USE_KEY"), "|", defined("ARRAY_FILTER_USE_BOTH"), "\n";
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
define("APP_NAME", "compiler");
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
$call = "defined";
echo $call("APP_NAME"), "|", $call("MISSING_CONST"), "\n";

function check_defined_inside_function() {
    define("INSIDE_DEFINED", 1);
    return defined("INSIDE_DEFINED") . ":" . defined("APP_NAME");
}

echo check_defined_inside_function(), "\n";
