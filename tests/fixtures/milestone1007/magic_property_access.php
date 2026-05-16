<?php
class Bag {
    public $name = "declared";

    public function __get($property) {
        echo "get:$property\n";
        if ($property === "count") {
            return 0;
        }
        return "value:" . $property;
    }

    public function __isset($property) {
        echo "isset:$property\n";
        return $property === "title" || $property === "count";
    }
}

$bag = new Bag();
echo $bag->name, "\n";
echo $bag->title, "\n";
echo isset($bag->title) ? "title:set\n" : "title:unset\n";
echo empty($bag->title) ? "title:empty\n" : "title:not-empty\n";
echo empty($bag->count) ? "count:empty\n" : "count:not-empty\n";
echo empty($bag->missing) ? "missing:empty" : "missing:not-empty";
