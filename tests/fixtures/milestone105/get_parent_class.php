<?php
class Box {}

$box = new box();
if (!get_parent_class($box)) {
    echo "object:false\n";
}
if (!get_parent_class("BOX")) {
    echo "string:false\n";
}

$call = "get_parent_class";
if (!$call($box)) {
    echo "dynamic:false";
}
