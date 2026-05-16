<?php
class Basket implements Countable {
    #[ReturnTypeWillChange]
    public function count($mode = null) {
        return 3;
    }
}

abstract class AbstractCounter implements Countable {}

$basket = new Basket();
echo is_countable($basket) ? "countable" : "plain";
echo "|";
echo count($basket);
