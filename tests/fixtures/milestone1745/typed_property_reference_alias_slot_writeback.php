<?php
class Box {
    public int $id = 1;
}

class Holder {
    public $items = array();
}

function assign_ref(&$value, $next) {
    $value = $next;
}

$box = new Box();
$alias =& $box->id;

$target = array();
$target["copy"] =& $alias;
$slot =& $target["copy"];
$slot = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($target["copy"]), ":", $target["copy"], "|", gettype($slot), ":", $slot, "\n";

assign_ref($target["copy"], "3");
echo gettype($box->id), ":", $box->id, "|", gettype($target["copy"]), ":", $target["copy"], "\n";

$nested = array("outer" => array());
$nested["outer"]["copy"] =& $alias;
$nestedAlias =& $nested["outer"]["copy"];
$nestedAlias = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($nested["outer"]["copy"]), ":", $nested["outer"]["copy"], "\n";

$holder = new Holder();
$holder->items["copy"] =& $alias;
$holderAlias =& $holder->items["copy"];
$holderAlias = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["copy"]), ":", $holder->items["copy"], "\n";

$property = "items";
$dynamicAlias =& $holder->{$property}["copy"];
$dynamicAlias = "6";
echo gettype($box->id), ":", $box->id, "|", gettype($holder->items["copy"]), ":", $holder->items["copy"], "\n";

$GLOBALS["copy"] =& $alias;
$globalAlias =& $GLOBALS["copy"];
$globalAlias = "7";
echo gettype($box->id), ":", $box->id, "|", gettype($GLOBALS["copy"]), ":", $GLOBALS["copy"], "|", gettype($globalAlias), ":", $globalAlias, "\n";

$outer = array();
$outer["arr"] =& $target;
$aliasRoot =& $outer["arr"];
$aliasRoot["copy"] = "8";
echo gettype($box->id), ":", $box->id, "|", gettype($aliasRoot["copy"]), ":", $aliasRoot["copy"], "|", gettype($target["copy"]), ":", $target["copy"];
