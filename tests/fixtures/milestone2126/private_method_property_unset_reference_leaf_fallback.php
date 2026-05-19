<?php
class Milestone2126_Box {
    private $items;

    public function set(&$source) {
        $this->items = ["leaf" => &$source];
    }

    public function &leaf() {
        return $this->items["leaf"];
    }

    public function drop() {
        unset($this->items);
    }
}

$source = "seed";
$box = new Milestone2126_Box();
$box->set($source);
$alias =& $box->leaf();
$box->drop();
$alias = "mutated";
echo $source, "|", $alias;
