<?php
class Box {
    public $items;
}

$source = "seed";
$box = new Box();
$box->items = ["leaf" => &$source];
$alias =& $box->items;
unset($box);
$alias["leaf"] = "mutated";
echo $source, "|", $alias["leaf"];
