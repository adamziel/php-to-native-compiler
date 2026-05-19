<?php
class Box {
    public $name;
}

$box = new Box();
$name = "name";
unset($box->$name["x"]->child);
