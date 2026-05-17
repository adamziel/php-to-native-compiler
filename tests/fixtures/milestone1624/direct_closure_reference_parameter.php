<?php
$counter = 0;
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};

echo $callback($option, "direct"), "|", $option, "|", $counter, "\n";
echo $callback($items["payload"]["slot"], "slot"), "|", $items["payload"]["slot"], "|", $counter;
