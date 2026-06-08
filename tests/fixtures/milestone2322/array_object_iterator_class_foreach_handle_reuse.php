<?php
class Milestone2322Iterator extends ArrayIterator {
    function rewind(): void {
        parent::rewind();
    }

    function valid(): bool {
        return parent::valid();
    }

    function current(): mixed {
        return parent::current();
    }

    function key(): string|int|null {
        return parent::key();
    }

    function next(): void {
        parent::next();
    }
}

class Milestone2322CapturingIterator extends Milestone2322Iterator {
    function rewind(): void {
        $GLOBALS["captured_iterator"] = $this;
        parent::rewind();
    }
}

$ao = new ArrayObject(array("a" => 1, "b" => 2), 0, "Milestone2322Iterator");
$first = $ao->getIterator();
echo spl_object_id($first), "\n";
foreach ($ao as $key => $value) {
}
$second = $ao->getIterator();
echo spl_object_id($second), "\n";

$capturing = new ArrayObject(array("x" => 1), 0, "Milestone2322CapturingIterator");
foreach ($capturing as $value) {
}
$after_capture = $capturing->getIterator();
echo spl_object_id($captured_iterator), "|", spl_object_id($after_capture);
