<?php
class Box {
    private $privateItems = ["slot" => "private"];
    protected $protectedItems = ["slot" => "protected"];

    public function run($key) {
        $private =& $this->privateItems[$key];
        $private = "private-alias";
        echo $this->privateItems[$key], "|";
        $this->privateItems[$key] = "private-property";
        echo $private, "|";

        $protected =& $this->protectedItems[$key];
        $protected = "protected-alias";
        echo $this->protectedItems[$key], "|";
        $this->protectedItems[$key] = "protected-property";
        echo $protected;
    }
}

$box = new Box();
$box->run("slot");

echo "|";

class Base {
    protected $items = ["slot" => "base"];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Child extends Base {
    public function aliasPeer($other, $key) {
        $alias =& $other->items[$key];
        $alias = "peer-alias";
        echo $other->readItem($key), "|";
        $other->items[$key] = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$peer = new Child();
$child->aliasPeer($peer, "slot");

echo "|";

class MaterializeBox {
    private $privateItems;
    protected $protectedItems = [];

    public function run($key) {
        $private =& $this->privateItems[$key];
        $private = "private-created";
        echo $this->privateItems[$key], "|";

        $protected =& $this->protectedItems[$key];
        $protected = "protected-created";
        echo $this->protectedItems[$key];
    }
}

$materialized = new MaterializeBox();
$materialized->run("slot");
