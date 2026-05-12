<?php
$items = [];
$items["first"] = "Ada";
$items[5] = "Bob";
$items["empty"] = "";
$items[] = "Linus";

$identity = array_map(null, $items);
print_r(array_keys($identity));
echo $identity["first"], "|", $identity[5], "|", $identity["empty"], "|", $identity[6], "\n";
$identity[] = "after";
echo $identity[7], "\n";
$identity["first"] = "Changed";
echo $items["first"], "|", $identity["first"], "\n";

$call = "array_map";
$dynamic = $call(null, ["x" => "A", 4 => "B"]);
print_r($dynamic);
