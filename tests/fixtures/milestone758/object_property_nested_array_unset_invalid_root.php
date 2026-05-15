<?php
class Bag {
    public $items;
}

$bag = new Bag();
$bag->items = "not-array";
unset($bag->items["key"]["child"]);

