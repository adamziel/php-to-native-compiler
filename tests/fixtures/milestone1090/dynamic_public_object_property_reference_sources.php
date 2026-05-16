<?php
class Box {
    public $value = "initial";
}

$box = new Box();
$property = "value";
$alias =& $box->$property;
$alias = "from-alias";
echo $box->value, "|";
$box->$property = "from-property";
echo $alias, "|";

$std = new stdClass();
$slot = "created";
$dynamic =& $std->$slot;
echo $dynamic === null ? "null" : "not-null";
echo "|";
$dynamic = "from-dynamic";
echo $std->created, "|";
$std->$slot = "from-slot";
echo $dynamic;
