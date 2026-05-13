<?php
$items = ["value" => 10, "float" => 1.5, "i" => 0, "sum" => 0];
++$items["value"];
echo $items["value"], "\n";
echo $items["value"]++, ":", $items["value"], "\n";
echo --$items["value"], ":", $items["value"], "\n";
echo $items["value"]--, ":", $items["value"], "\n";

echo $items["float"]++, ":", $items["float"], "\n";
echo --$items["float"], ":", $items["float"], "\n";

$items["value"] = 1;
++$items["value"] + 10;
echo "side:", $items["value"], "\n";
$items["value"]++ + 10;
echo "side:", $items["value"], "\n";

for ($items["i"] = 0; $items["i"] < 3; $items["i"]++) {
    $items["sum"] += $items["i"];
}
echo $items["sum"], ":", $items["i"];
