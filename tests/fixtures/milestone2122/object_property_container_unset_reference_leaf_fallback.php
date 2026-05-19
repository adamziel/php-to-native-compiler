<?php
class Box {
    public $items;
}

$source = "seed";
$box = new Box();
$box->items = ["box" => ["leaf" => &$source]];
$alias =& $box->items["box"];
unset($box->items);
$alias["leaf"] = "mutated";

$other = "start";
$box->items = ["box" => ["leaf" => &$other]];
$copy =& $box->items["box"];
$box->items = [];
$copy["leaf"] = "changed";

echo $source, "|", $alias["leaf"], "|", $other, "|", $copy["leaf"], "|", count($box->items);
