<?php
class Box {
    public $items;

    public function set(&$source) {
        $this->items = ["leaf" => &$source];
    }

    public function &leaf() {
        return $this->items["leaf"];
    }

    public function __unset($name) {
        unset($this->items);
    }
}

$source = "seed";
$box = new Box();
$box->set($source);
$alias =& $box->leaf();
unset($box->missing);
$alias = "mutated";
echo $source, "|", $alias;
