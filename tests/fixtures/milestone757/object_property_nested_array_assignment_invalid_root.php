<?php
class Bag {
    public $items;
}

$bag = new Bag();
$bag->items = "not-array";
$bag->items["key"]["child"] = "value";

