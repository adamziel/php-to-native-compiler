<?php
class Milestone1750_UnsetArrayReferenceBox {
    public int $id = 1;
}

class Milestone1750_UnsetArrayReferenceBag {
    public $items = array();
}

$box = new Milestone1750_UnsetArrayReferenceBox();
$alias =& $box->id;

$items = array();
$items["copy"] =& $alias;
$slot =& $items["copy"];
unset($items["copy"]);
$slot = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($slot), ":", $slot, "\n";

$items["outer"] = array();
$items["outer"]["copy"] =& $alias;
$nested =& $items["outer"]["copy"];
unset($items["outer"]);
$nested = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($nested), ":", $nested, "\n";

$bag = new Milestone1750_UnsetArrayReferenceBag();
$bag->items["copy"] =& $alias;
$propertySlot =& $bag->items["copy"];
unset($bag->items["copy"]);
$propertySlot = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($propertySlot), ":", $propertySlot;
