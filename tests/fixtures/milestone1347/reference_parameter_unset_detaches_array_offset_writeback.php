<?php
function mutate_then_detach(&$slot, $label) {
    $slot = $label . ":mutated";
    unset($slot);
    $slot = $label . ":local";
}

$_REQUEST["payload"] = ["slot" => "request:seed"];
mutate_then_detach($_REQUEST["payload"]["slot"], "request");
echo $_REQUEST["payload"]["slot"];
echo "\n";

$GLOBALS["bag"] = ["slot" => "global:seed"];
mutate_then_detach($GLOBALS["bag"]["slot"], "global");
echo $GLOBALS["bag"]["slot"];
echo "\n";

$items = ["outer" => ["slot" => "array:seed"]];
mutate_then_detach($items["outer"]["slot"], "array");
echo $items["outer"]["slot"];
