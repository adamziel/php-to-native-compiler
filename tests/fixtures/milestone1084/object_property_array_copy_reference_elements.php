<?php
class Box {
    public $items = [];
}

$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$copy = $box->items;
$copy["slot"] = "b";
echo $value, "|", $box->items["slot"], "|", $copy["slot"], "\n";

$box = new Box();
$box->items = ["slot" => "x"];
$other = "x";
$box->items["slot"] =& $other;
$copy = $box->items;
$other = "y";
echo $copy["slot"], "|", $box->items["slot"];
$copy["slot"] = "z";
echo "|", $other, "|", $box->items["slot"], "\n";

$box = new Box();
$box->items = ["slot" => "x"];
$copy = $box->items;
$copy["slot"] = "b";
echo $box->items["slot"], "|", $copy["slot"], "\n";

$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$box->items = ["slot" => "new"];
$copy = $box->items;
$copy["slot"] = "b";
echo $value, "|", $box->items["slot"], "|", $copy["slot"];
