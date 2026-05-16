<?php
class Box {
    public $value = "initial";
    public $items = ["first"];
}

$box = new Box();
$alias =& $box->value;
$alias = "value-alias";
echo $box->value, "\n";
$box->value = "value-property";
echo $alias, "\n";

$items =& $box->items;
$items[0] = "array-alias";
echo $box->items[0], "\n";
$box->items = ["array-property"];
echo $items[0];
