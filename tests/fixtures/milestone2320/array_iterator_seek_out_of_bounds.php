<?php
$it = new ArrayIterator(array(0, 1, 2));
$it->seek(1);
echo $it->key(), ":", $it->current(), "\n";

try {
    $it->seek(-1);
} catch (OutOfBoundsException $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

try {
    $it->seek(3);
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$it->seek(2);
echo $it->key(), ":", $it->current();
