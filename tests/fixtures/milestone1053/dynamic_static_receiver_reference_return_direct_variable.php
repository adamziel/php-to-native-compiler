<?php
class Box {
    public static function &identity(&$value) {
        return $value;
    }
}

$class = "Box";
$value = 1;
$alias =& $class::identity($value);
$alias = 2;
echo "class=", $value, "|";

$box = new Box();
$value = 3;
$alias =& $box::identity($value);
$alias = 4;
echo "object=", $value;
