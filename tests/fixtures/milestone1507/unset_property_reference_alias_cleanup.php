<?php
class WP_RefCow_Unset_Property_Bag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
    private $privateItems = ["slot" => "private-seed", "outer" => ["leaf" => "private-nested"]];

    public function clearPrivate() {
        $alias =& $this->privateItems["slot"];
        $leaf =& $this->privateItems["outer"]["leaf"];

        unset($this->privateItems);
        echo isset($this->privateItems) ? "private:set" : "private:unset";
        echo "|alias=", $alias, "|leaf=", $leaf, "\n";
        $alias = "private-after";
        $leaf = "private-changed";
        echo isset($this->privateItems) ? "private:set" : "private:unset";
        echo "|alias=", $alias, "|leaf=", $leaf;
    }
}

$bag = new WP_RefCow_Unset_Property_Bag();
$alias =& $bag->items["slot"];
$leaf =& $bag->items["outer"]["leaf"];

unset($bag->items);
echo isset($bag->items) ? "public:set" : "public:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";
$alias = "after";
$leaf = "changed";
echo isset($bag->items) ? "public:set" : "public:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";

$bag->clearPrivate();
