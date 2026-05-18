<?php
error_reporting(0);

class Milestone1759_Box {
    public int $id = 1;
}

class Milestone1759_Holder {
    public $items = false;
    public $appendItems = false;
}

$box = new Milestone1759_Box();
$alias =& $box->id;

$direct = false;
$direct["leaf"] = array("copy" => &$alias);
$direct["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($direct["leaf"]["copy"]), ":", $direct["leaf"]["copy"], "\n";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] = array("copy" => &$alias);
$GLOBALS["globalRoot"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($globalRoot["leaf"]["copy"]), ":", $globalRoot["leaf"]["copy"], "\n";

$holder = new Milestone1759_Holder();
$holder->items["leaf"] = array("copy" => &$alias);
$holder->items["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["leaf"]["copy"]), ":", $holder->items["leaf"]["copy"], "\n";

$append = false;
$append["bucket"][] = array("copy" => &$alias);
$append["bucket"][0]["copy"] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($append["bucket"][0]["copy"]), ":", $append["bucket"][0]["copy"], "\n";

$holder->appendItems["bucket"][] = array("copy" => &$alias);
$holder->appendItems["bucket"][0]["copy"] = "6";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->appendItems["bucket"][0]["copy"]), ":", $holder->appendItems["bucket"][0]["copy"];
