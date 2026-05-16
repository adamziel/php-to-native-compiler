<?php
function &identity(&$value) {
    return $value;
}

$value = 1;
$alias =& identity($value);
$alias = 2;
echo "value=", $value, "|";

$value = 3;
echo "alias=", $alias, "\n";

unset($alias);
$alias = 9;
echo "detached=", $alias, "|value=", $value;
