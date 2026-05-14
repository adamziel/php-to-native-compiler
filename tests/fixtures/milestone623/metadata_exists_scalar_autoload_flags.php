<?php
class Box {}

var_dump(class_exists("BOX", 1));
var_dump(class_exists("Missing", 0));
var_dump(interface_exists("Box", "1"));
var_dump(trait_exists("Box", "0"));
var_dump(enum_exists("Box", 0.5));

$call = "class_exists";
var_dump($call("box", "false"));
