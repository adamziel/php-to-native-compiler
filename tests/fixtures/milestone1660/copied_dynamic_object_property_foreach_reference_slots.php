<?php
class Milestone1660Box {
    public $items;
}

$box = new Milestone1660Box();
$name = "items";
$box->items = array("outer" => array("plain" => "p", "slot" => "orig"));
$alias =& $box->{$name}["outer"]["slot"];
$copy = $box->{$name}["outer"];

foreach ($copy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->items["outer"]["slot"], "|", $copy["slot"], "|", $box->items["outer"]["plain"], "|", $copy["plain"], "|";
$copy["slot"] = "direct";
echo $value, "|";
$value = "tail";
echo $box->items["outer"]["slot"], "|", $copy["slot"], "\n";
unset($value);

$box->items = array("plain" => "root", "slot" => "whole");
$rootAlias =& $box->{$name}["slot"];
$rootCopy = $box->{$name};
foreach ($rootCopy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->items["slot"], "|", $rootCopy["slot"], "|", $box->items["plain"], "|", $rootCopy["plain"], "|";
$rootCopy["slot"] = "root-direct";
echo $value, "|";
$value = "root-tail";
echo $box->items["slot"], "|", $rootCopy["slot"];
