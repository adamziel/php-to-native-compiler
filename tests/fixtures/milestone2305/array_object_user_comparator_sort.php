<?php
error_reporting(0);
function descending($left, $right) {
    if ($left == $right) {
        return 0;
    }
    return $left < $right ? 1 : -1;
}

$ao = new ArrayObject(array(2, 3, 1));
$ao->uasort('descending');
foreach ($ao->getArrayCopy() as $key => $value) {
    echo "ao-value:$key:$value\n";
}
$ao->uksort('descending');
foreach ($ao->getArrayCopy() as $key => $value) {
    echo "ao-key:$key:$value\n";
}

$it = new ArrayIterator(array('b' => 2, 'a' => 3, 'c' => 1));
$it->uasort('descending');
foreach ($it as $key => $value) {
    echo "it-value:$key:$value\n";
}
$it->uksort('descending');
foreach ($it as $key => $value) {
    echo "it-key:$key:$value\n";
}

$std = new stdClass();
$std->b = 2;
$std->a = 3;
$std->c = 1;
$wrapped = new ArrayObject($std);
$wrapped->uasort('descending');
foreach ($wrapped->getArrayCopy() as $key => $value) {
    echo "obj-value:$key:$value\n";
}
$wrapped->uksort('descending');
foreach ($wrapped->getArrayCopy() as $key => $value) {
    echo "obj-key:$key:$value\n";
}

try {
    $ao->uasort();
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}
