<?php
$value = 2;
$copy = ($value += 3);
echo $copy, ":", $value, "\n";

$items = ["count" => 4];
echo ($array_copy = ($items["count"] *= 2)), ":", $array_copy, ":", $items["count"], "\n";

class Box {
    public $count;
}

$box = new Box();
$box->count = 5;
echo ($property_copy = ($box->count -= 1)), ":", $property_copy, ":", $box->count, "\n";

echo ($fallback_copy = ($missing ??= "fallback")), ":", $fallback_copy, ":", $missing, "\n";

function should_not_run() {
    echo "rhs\n";
    return "new";
}

$kept = "old";
echo ($kept_copy = ($kept ??= should_not_run())), ":", $kept_copy, ":", $kept, "\n";

$slots = [];
echo ($slot_copy = ($slots["name"] ??= "Ada")), ":", $slot_copy, ":", $slots["name"];
