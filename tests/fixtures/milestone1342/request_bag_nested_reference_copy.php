<?php
function handle_request_payload() {
    $_REQUEST["payload"] = ["a" => "one"];
    $alias =& $_REQUEST["payload"]["a"];
    $copy = $_REQUEST["payload"];
    $alias = "two";
    echo $_REQUEST["payload"]["a"], "|", $copy["a"], "|";
    $copy["a"] = "three";
    echo $alias, "|", $_REQUEST["payload"]["a"];
}

handle_request_payload();
echo "\n";

$items = ["outer" => ["slot" => "alpha"]];
$slot =& $items["outer"]["slot"];
$outer = $items["outer"];
$slot = "beta";
echo $items["outer"]["slot"], "|", $outer["slot"], "|";
$outer["slot"] = "gamma";
echo $slot, "|", $items["outer"]["slot"];
