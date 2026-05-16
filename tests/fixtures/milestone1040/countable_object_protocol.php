<?php
class Basket implements Countable {
    private $items;

    public function __construct($items) {
        $this->items = $items;
    }

    #[ReturnTypeWillChange]
    public function count() {
        return count($this->items);
    }
}

class Box {}

$basket = new Basket([1, 2, 3]);
$box = new Box();
echo is_countable($basket) ? "countable" : "plain";
echo "|";
echo count($basket);
echo "|";
echo is_countable($box) ? "countable" : "plain";
echo "|";
echo count([1, 2]);
