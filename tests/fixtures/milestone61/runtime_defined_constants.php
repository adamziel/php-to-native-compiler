<?php
define("APP_NAME", "compiler");
echo define("APP_VERSION", 1), "\n";
echo constant("APP_NAME"), "|", constant("APP_VERSION"), "\n";

$items = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = constant("APP_ITEMS");
$copy["name"] = "changed";
$again = constant("APP_ITEMS");
echo count($copy), "|", $copy["name"], "|", $again["name"], "|", $again["nested"]["x"], "\n";

function constant_scope() {
    define("INSIDE_FUNCTION", "inside");
    return constant("APP_NAME") . ":" . constant("INSIDE_FUNCTION");
}

echo constant_scope(), "\n";
$call = "define";
echo $call("DYNAMIC_NAME", "dynamic"), "\n";
echo constant("DYNAMIC_NAME"), "\n";
