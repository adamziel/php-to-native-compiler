<?php
error_reporting(0);

class Milestone1760_Box {
    public int $id = 1;
}

class Milestone1760_Holder {
    public $items = false;
}

$box = new Milestone1760_Box();
$alias =& $box->id;

$direct = false;
$direct["leaf"] =& $alias;
$direct["leaf"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($direct["leaf"]), ":", $direct["leaf"], "\n";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] =& $alias;
$GLOBALS["globalRoot"]["leaf"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($globalRoot["leaf"]), ":", $globalRoot["leaf"], "\n";

$holder = new Milestone1760_Holder();
$holder->items["leaf"] =& $alias;
$holder->items["leaf"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["leaf"]), ":", $holder->items["leaf"], "\n";

$append = false;
$append[] =& $alias;
$append[0] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($append[0]), ":", $append[0];
