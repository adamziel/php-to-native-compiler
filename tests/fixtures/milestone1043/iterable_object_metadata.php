<?php
class Plain {}

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
        return new ArrayIterator([]);
    }
}

$values = [null, false, true, 0, 3.5, "", [], [1], new Plain(), new CustomIterator(), new Aggregate()];
foreach ($values as $value) {
    echo is_iterable($value) ? "1" : "0";
}
