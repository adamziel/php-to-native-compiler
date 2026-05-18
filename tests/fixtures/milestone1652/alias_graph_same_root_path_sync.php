<?php
$items = array("root" => array("slot" => "old"), "source" => "source-old");
$root =& $items["root"];
$source =& $items["source"];
$same =& $source;
$root["slot"] =& $source;
$source = "source-write";
echo $items["root"]["slot"], "|", $same, "|";
$items["root"]["slot"] = "slot-write";
echo $source, "|", $same, "|", $items["source"], "\n";

$_REQUEST["payload"] = array("root" => array("slot" => "old"), "source" => "request-old");
$requestRoot =& $_REQUEST["payload"]["root"];
$requestSource =& $_REQUEST["payload"]["source"];
$requestSame =& $requestSource;
$requestRoot["slot"] =& $requestSource;
$requestSource = "request-source";
echo $_REQUEST["payload"]["root"]["slot"], "|", $requestSame, "|";
$_REQUEST["payload"]["root"]["slot"] = "request-slot";
echo $requestSource, "|", $requestSame, "|", $_REQUEST["payload"]["source"], "\n";

class AliasGraphBox {
    public $items = array("root" => array("slot" => "old"), "source" => "box-old");
}

$box = new AliasGraphBox();
$boxRoot =& $box->items["root"];
$boxSource =& $box->items["source"];
$boxSame =& $boxSource;
$boxRoot["slot"] =& $boxSource;
$boxSource = "box-source";
echo $box->items["root"]["slot"], "|", $boxSame, "|";
$box->items["root"]["slot"] = "box-slot";
echo $boxSource, "|", $boxSame, "|", $box->items["source"];
