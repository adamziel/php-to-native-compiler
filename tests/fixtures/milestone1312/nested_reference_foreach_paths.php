<?php
$items = ["outer" => ["a" => "one", "b" => "two"]];

foreach ($items["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $items["outer"]["c"] = "three";
    }
}

echo $items["outer"]["a"], "|", $items["outer"]["b"], "|", $items["outer"]["c"], "|", $value, "\n";
$value = "tail";
echo $items["outer"]["c"], "|", $value, "\n";
unset($value);

$GLOBALS["bag"] = ["child" => ["x" => "ex", "y" => "why"]];
foreach ($GLOBALS["bag"]["child"] as $key => &$value) {
    $value = $key . "=" . $value;
}
echo $GLOBALS["bag"]["child"]["x"], "|", $GLOBALS["bag"]["child"]["y"], "|", $value, "\n";
unset($value);

$_REQUEST["payload"] = ["first" => "alpha", "second" => "beta"];
function mutate_request_payload() {
    foreach ($_REQUEST["payload"] as $key => &$value) {
        if ($key === "second") {
            $value = "changed";
        }
    }
    unset($value);
}
mutate_request_payload();
echo $_REQUEST["payload"]["first"], "|", $_REQUEST["payload"]["second"];
