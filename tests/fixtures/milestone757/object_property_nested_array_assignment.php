<?php
class Bag {
    public $items;
}

$bag = new Bag();
$file = "translation.mo";
$locale = "en_US";
$domain = "default";
$bag->items[$file][$locale][$domain] = "loaded";
$bag->items[$file]["fr_FR"]["default"] = "charge";
$bag->items[$file][$locale][] = "fallback";
$bag->items[] = "root-append";
echo $bag->items[$file][$locale][$domain], "\n";
echo $bag->items[$file]["fr_FR"]["default"], "\n";
echo $bag->items[$file][$locale][0], "\n";
echo $bag->items[0], "\n";
