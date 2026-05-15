<?php
class Box {
    public $name;
    public $child;
}

$child = new Box();
$child->name = "child";
$box = new Box();
$box->name = "original";
$box->child = $child;

$copy = clone $box;
$copy->name = "copy";
$box_child = $box->child;
$copy_child = $copy->child;

var_dump($box === $copy);
var_dump(spl_object_id($box) === spl_object_id($copy));
echo $box->name, "\n";
echo $copy->name, "\n";
var_dump($box_child === $copy_child);
