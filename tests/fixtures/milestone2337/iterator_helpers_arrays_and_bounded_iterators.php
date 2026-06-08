<?php
$array = array("a" => 1, "b" => 2, 5 => 3);
print_r(iterator_to_array($array));
print_r(iterator_to_array($array, false));
echo "count-array=", iterator_count($array), "\n";

$it = new ArrayIterator(array("x" => "ex", "y" => "why"));
print_r(iterator_to_array($it));
echo "after-arrayiterator=", $it->valid() ? "valid" : "invalid", "\n";

$again = new ArrayIterator(array("x" => "ex", "y" => "why"));
print_r(iterator_to_array($again, false));

$ao = new ArrayObject(array("p" => 7, "q" => 8));
print_r(iterator_to_array($ao, false));

$counted = new ArrayIterator(array(10, 20));
echo "count-iterator=", iterator_count($counted), "|", ($counted->valid() ? "valid" : "invalid"), "\n";

try {
    iterator_count("bad");
} catch (Throwable $e) {
    echo $e::class, ":", $e->getMessage(), "\n";
}

try {
    iterator_to_array(array(1), array());
} catch (Throwable $e) {
    echo $e::class, ":", $e->getMessage();
}
