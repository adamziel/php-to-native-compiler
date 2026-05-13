<?php
$left = $right = 10;
echo $left, ":", $right, "\n";

echo ($outer = $inner = 20), ":", $outer, ":", $inner, "\n";

$items = [];
echo ($copy = $items["name"] = "Ada"), ":", $copy, ":", $items["name"], "\n";

class Box {
    public $value;
}

$box = new Box();
echo ($same = $box->value = "stored"), ":", $same, ":", $box->value;
