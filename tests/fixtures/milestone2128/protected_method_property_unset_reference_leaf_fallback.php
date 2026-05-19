<?php
class Milestone2128_Base {
    protected $items;

    public function set(&$source) {
        $this->items = ["leaf" => &$source];
    }

    public function &leaf() {
        return $this->items["leaf"];
    }
}

class Milestone2128_Child extends Milestone2128_Base {
    public function drop() {
        unset($this->items);
    }
}

$source = "seed";
$box = new Milestone2128_Child();
$box->set($source);
$alias =& $box->leaf();
$box->drop();
$alias = "mutated";
echo $source, "|", $alias;
