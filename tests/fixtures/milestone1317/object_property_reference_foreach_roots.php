<?php
class Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

$bag = new Bag();
foreach ($bag->items["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}

echo $bag->items["outer"]["a"], "|", $bag->items["outer"]["b"], "|", $bag->items["outer"]["c"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $bag->items["outer"]["c"], "|", $value, "\n";
unset($value);

foreach ($bag->items as $key => &$value) {
    if ($key === "outer") {
        $value["d"] = "delta";
    }
}
unset($value);
echo $bag->items["outer"]["d"];
