<?php
$_REQUEST["payload"] = array("slot" => "request-old", "append" => array());
$payload =& $_REQUEST["payload"];
$value = "request-new";
$payload["slot"] =& $value;
$value = "request-source";
echo $_REQUEST["payload"]["slot"], "|";
$_REQUEST["payload"]["slot"] = "request-target";
echo $value, "\n";

class ReferenceAssignmentBox {
    public $items = array("slot" => "box-old", "nested" => array("slot" => "nested-old"));
}

$box = new ReferenceAssignmentBox();
$slot =& $box->items["slot"];
$boxValue = "box-new";
$slot =& $boxValue;
$boxValue = "box-source";
echo $box->items["slot"], "|";
$box->items["slot"] = "box-target";
echo $boxValue, "\n";

$nested =& $box->items["nested"];
$nestedValue = "nested-new";
$nested["slot"] =& $nestedValue;
$nestedValue = "nested-source";
echo $box->items["nested"]["slot"], "|";
$box->items["nested"]["slot"] = "nested-target";
echo $nestedValue;
