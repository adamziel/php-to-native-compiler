<?php
class Milestone1906_Bag implements ArrayAccess {
    public $items = array();
    public $trace = array();

    private function &pick($offset) {
        $this->trace[] = "pick:" . $offset;
        if ($offset === "slot") {
            return $this->items[$offset];
        }
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get:" . $offset;
        return $this->pick($offset);
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1906_Box {
    public $bags = array();
    public $trace = array();

    public function __construct($bag) {
        $this->bags["outer"] = $bag;
    }

    public function __get($name) {
        $this->trace[] = "get:" . $name;
        return $this->bags[$name];
    }
}

$bag = new Milestone1906_Bag();
$bag->items["slot"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $bag->items["slot"]["leaf"];
$box = new Milestone1906_Box($bag);

$copy = $box->outer["slot"];
$copy["leaf"] = "changed";
$copy["plain"]["value"] = "copy-changed";

echo $alias, "|", $bag->items["slot"]["leaf"], "|", $bag->items["slot"]["plain"]["value"], "|", implode(",", $box->trace), "|", implode(",", $bag->trace);
