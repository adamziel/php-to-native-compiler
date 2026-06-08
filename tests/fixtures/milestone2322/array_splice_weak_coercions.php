<?php
$items = [0, 1, 2, 3];
$removed = array_splice($items, "2.7", "1.1", "x");
echo implode(",", $removed), "|", implode(",", $items), "\n";

$items = [0, 1, 2, 3];
$removed = array_splice($items, false, true, "x");
echo implode(",", $removed), "|", implode(",", $items), "\n";

$items = [0, 1, 2, 3];
$removed = array_splice($items, null, null, "x");
echo implode(",", $removed), "|", implode(",", $items), "\n";

$call = "array_splice";
$items = [0, 1, 2, 3];
$removed = $call($items, true, false, [9]);
echo count($removed), "|", implode(",", $items), "\n";

try {
    array_splice($items, [], 1);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_splice($items, 1, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
