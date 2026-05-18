<?php
function milestone1653_mark(&$value) {
    $value = $value . ":mark";
}

$value = "seed";
$args = array(&$value);
$registry = [];
$registry["args"] = $args;
call_user_func_array("milestone1653_mark", $registry["args"]);
echo "array-copy=", $value, "|", $registry["args"][0], "\n";

$registry["args"] = ["fresh"];
$value = "changed";
echo "array-detach=", $registry["args"][0], "|", $value, "\n";

class Milestone1653_Store {
    public $groups = [];
}

$propertyValue = "property";
$propertyArgs = array(&$propertyValue);
$store = new Milestone1653_Store();
$store->groups["args"] = $propertyArgs;
call_user_func_array("milestone1653_mark", $store->groups["args"]);
echo "property-copy=", $propertyValue, "|", $store->groups["args"][0], "\n";

$store->groups["args"] = ["property-fresh"];
$propertyValue = "property-changed";
echo "property-detach=", $store->groups["args"][0], "|", $propertyValue;
