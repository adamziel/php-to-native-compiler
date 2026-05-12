<?php
define("APP_NAME", "compiler");
define("APP_VERSION", 2);
echo APP_NAME, "|", APP_VERSION + 3, "\n";
echo ARRAY_FILTER_USE_KEY, "|", ARRAY_FILTER_USE_BOTH, "\n";

$items = ["name" => "Ada", "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "|", $copy["name"], "\n";

function read_constant_inside_function() {
    define("FUNCTION_CONSTANT", "inside");
    return APP_NAME . ":" . FUNCTION_CONSTANT;
}

echo read_constant_inside_function(), "\n";
$call = "define";
$call("DYNAMIC_CONSTANT", "dynamic");
echo DYNAMIC_CONSTANT, "\n";
