<?php
class Bag {
    public $items;
    public $empty;
}

$bag = new Bag();
$file = "translation.mo";
$locale = "en_US";
$domain = "default";
$bag->items[$file][$locale][$domain] = "loaded";
$bag->items[$file][$locale]["fallback"] = "fallback";
$bag->items[$file]["fr_FR"][$domain] = "charge";
unset($bag->items[$file][$locale][$domain]);
unset($bag->items["missing"]["path"]);
unset($bag->empty["path"]);
echo $bag->items[$file][$locale]["fallback"], "\n";
echo $bag->items[$file]["fr_FR"][$domain], "\n";

