<?php
class Box {
    private $privateItems = [];
    protected $protectedItems = [];

    public function run($outer) {
        $private =& $this->privateItems[];
        $private = "private-alias";
        echo $this->privateItems[0], "|";
        $this->privateItems[0] = "private-property";
        echo $private, "|";

        $protected =& $this->protectedItems[];
        $protected = "protected-alias";
        echo $this->protectedItems[0], "|";
        $this->protectedItems[0] = "protected-property";
        echo $protected, "|";

        $nested =& $this->protectedItems[$outer][];
        $nested = "nested-created";
        echo $this->protectedItems[$outer][0];
    }
}

$box = new Box();
$box->run("outer");

echo "|";

class Base {
    protected $items = [];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Child extends Base {
    public function aliasPeer($other) {
        $alias =& $other->items[];
        $alias = "peer-alias";
        echo $other->readItem(0), "|";
        $other->items[0] = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$peer = new Child();
$child->aliasPeer($peer);
