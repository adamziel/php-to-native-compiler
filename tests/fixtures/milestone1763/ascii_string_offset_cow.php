<?php
$s = "abc";
$copy = $s;
$s[1] = "X";
echo $s, "|", $copy, "|";

$alias =& $s;
$alias[2] = "Y";
echo $s, "|", $alias, "|";

$s[5] = "Z";
echo $s, "|", $s[0], "|", $s[5], "|";

$s[-1] = "Q";
echo $s, "|", $s[-1];

class Milestone1763_Box {
    public $text = "cat";
}

$box = new Milestone1763_Box();
$propertyCopy = $box->text;
$box->text[1] = "U";
echo "|", $box->text, "|", $propertyCopy;

$propertyAlias =& $box->text;
$propertyAlias[2] = "T";
echo "|", $box->text, "|", $propertyAlias;

$items = ["name" => "abcd"];
$itemsCopy = $items;
$items["name"][1] = "X";
echo "|", $items["name"], "|", $itemsCopy["name"], "|", $items["name"][1];

$itemAlias =& $items["name"];
$itemAlias[2] = "Y";
echo "|", $items["name"], "|", $itemAlias;

$numeric = "wxyz";
$numericCopy = $numeric;
$numeric["01"] = "A";
$numeric["+2"] = "B";
echo "|", $numeric, "|", $numericCopy, "|", $numeric[" 1"];

$box->text = "dog";
$box->text["02"] = "G";
echo "|", $box->text;

$items["name"] = "rust";
$items["name"][" 2"] = "S";
echo "|", $items["name"];
