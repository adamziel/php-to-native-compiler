<?php
class Box {
    public function &identity(&$value) {
        return $value;
    }
}

$box = new Box();
$value = 1;
$alias =& $box->identity($value);
$alias = 2;
echo "value=", $value, "|";

$value = 3;
echo "alias=", $alias, "\n";

unset($alias);
$alias = 9;
echo "detached=", $alias, "|value=", $value;
