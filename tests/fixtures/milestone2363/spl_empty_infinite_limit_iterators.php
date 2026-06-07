<?php
echo "empty\n";
foreach (new LimitIterator(new EmptyIterator(), 0, 3) as $key => $value) {
    echo "$key=>$value\n";
}

echo "infinite\n";
$it = new ArrayIterator(array(0 => "A", 1 => "B", 2 => "C", 3 => "D"));
$it = new LimitIterator(new InfiniteIterator($it), 2, 5);
foreach ($it as $key => $value) {
    echo "$key=>$value\n";
}

echo "nested\n";
$it = new ArrayIterator(array(0 => "A", 1 => "B", 2 => "C", 3 => "D"));
$it = new LimitIterator(new InfiniteIterator(new LimitIterator($it, 1, 2)), 2, 5);
foreach ($it as $key => $value) {
    echo "$key=>$value\n";
}

try {
    new LimitIterator(new ArrayIterator(array(1)), -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}

try {
    foreach (new LimitIterator(new ArrayIterator(array("x")), 3) as $value) {
        echo $value;
    }
} catch (OutOfBoundsException $e) {
    echo $e->getMessage();
}
