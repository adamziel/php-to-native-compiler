<?php
$storage = array("nested" => array("base" => "keep"));

class Milestone1668MagicAppendBox {
    public function &__get($name) {
        echo "get:$name\n";
        global $storage;
        return $storage;
    }
}

$holders = array("box" => new Milestone1668MagicAppendBox());
$alias =& $holders["box"]->missing[];
$alias = "first";
echo $storage[0], "|", $alias, "\n";
$storage[0] = "store";
echo $alias, "|";
$alias = "tail";
echo $storage[0], "\n";
unset($alias);

$nested =& $holders["box"]->missing["nested"][];
$nested = "child";
echo $storage["nested"][0], "|", $nested, "\n";

$property = "dynamicMissing";
$dynamic =& $holders["box"]->{$property}[];
$dynamic = "dynamic";
echo $storage[1], "|", $dynamic;
