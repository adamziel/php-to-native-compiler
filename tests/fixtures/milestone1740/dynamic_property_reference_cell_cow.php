<?php
error_reporting(0);

function milestone1740_box() {
    static $box;
    if (!$box) {
        $box = new stdClass();
    }
    return $box;
}

$box = new stdClass();
$alias =& $box->created;
echo ($alias === null ? "null" : "not-null"), "|";
$alias = "alias";
echo $box->created, "|";
$copy = clone $box;
$copy->created = "clone";
echo $alias, "|", $box->created, "|", $copy->created, "\n";

$property = "dynamic";
$dynamic =& $box->{$property};
$dynamic = "dynamic-alias";
echo $box->dynamic, "|";
$box->{$property} = "dynamic-property";
echo $dynamic, "\n";

$expr =& milestone1740_box()->created;
$expr = "expr";
echo milestone1740_box()->created, "|";
$exprCopy = clone milestone1740_box();
$exprCopy->created = "expr-clone";
echo $expr, "|", milestone1740_box()->created, "|", $exprCopy->created, "\n";

$arrayBox = new stdClass();
$arrayAlias =& $arrayBox->created["slot"];
$arrayAlias = "array-alias";
echo $arrayBox->created["slot"], "|";
$arrayCopy = clone $arrayBox;
$arrayCopy->created["slot"] = "array-clone";
echo $arrayAlias, "|", $arrayBox->created["slot"], "|", $arrayCopy->created["slot"], "\n";

$appendBox = new stdClass();
$appendAlias =& $appendBox->created[];
$appendAlias = "append-alias";
echo $appendBox->created[0], "|";
$appendCopy = clone $appendBox;
$appendCopy->created[0] = "append-clone";
echo $appendAlias, "|", $appendBox->created[0], "|", $appendCopy->created[0];
