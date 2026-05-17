<?php
$GLOBALS["bag"] = ["one" => "a", "two" => "b"];

foreach ($GLOBALS["bag"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "one") {
        $GLOBALS["bag"]["three"] = "c";
    }
}

echo $GLOBALS["bag"]["one"], "|", $GLOBALS["bag"]["two"], "|", $GLOBALS["bag"]["three"], "|", $value, "|", $key, "\n";
$GLOBALS["bag"]["three"] = "direct";
echo $value, "|";
$value = "tail";
echo $GLOBALS["bag"]["three"], "|", $value, "\n";
unset($value);

function mutate_global_bag() {
    foreach ($GLOBALS["bag"] as $key => &$value) {
        if ($key === "one") {
            $value = "fn";
        }
    }
    unset($value);
}

mutate_global_bag();
echo $GLOBALS["bag"]["one"], "|", $GLOBALS["bag"]["two"], "|", $GLOBALS["bag"]["three"];
