<?php
class Box {
    public int $id;
}

class Holder {
    public $items = array();
}

$box = new Box();
$box->id = 1;
$alias =& $box->id;

$target = array();
$target["copy"] =& $alias;
$target["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($target["copy"]), ":", $target["copy"], "\n";

$nested = array("outer" => array());
$nested["outer"]["copy"] =& $alias;
$nested["outer"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($nested["outer"]["copy"]), ":", $nested["outer"]["copy"], "\n";

$holder = new Holder();
$holder->items["copy"] =& $alias;
$holder->items["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["copy"]), ":", $holder->items["copy"], "\n";

$target["copy"] += 1;
echo gettype($box->id), ":", $box->id, "|", gettype($target["copy"]), ":", $target["copy"];
