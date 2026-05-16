<?php
class CustomIterator implements Iterator {
    #[ReturnTypeWillChange]
    public function current() {
        return null;
    }

    #[ReturnTypeWillChange]
    public function key() {
        return null;
    }

    #[ReturnTypeWillChange]
    public function next() {
        return null;
    }

    #[ReturnTypeWillChange]
    public function rewind() {
        return null;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return false;
    }
}

class Aggregate implements IteratorAggregate {
    #[ReturnTypeWillChange]
    public function getIterator() {
        return null;
    }
}

$iterator = new CustomIterator();
$aggregate = new Aggregate();

echo is_iterable($iterator) ? "iterator" : "plain";
echo "|";
echo is_iterable($aggregate) ? "aggregate" : "plain";
