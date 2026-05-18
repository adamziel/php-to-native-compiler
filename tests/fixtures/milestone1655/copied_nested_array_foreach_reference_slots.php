<?php
$items = array("outer" => array("plain" => "p", "slot" => "orig"));
$alias =& $items["outer"]["slot"];
$copy = $items["outer"];

foreach ($copy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $items["outer"]["slot"], "|", $copy["slot"], "|", $items["outer"]["plain"], "|", $copy["plain"], "|";
$copy["slot"] = "direct";
echo $value, "|";
$value = "tail";
echo $items["outer"]["slot"], "|", $copy["slot"], "\n";
unset($value);

$_REQUEST["payload"] = array("plain" => "r", "slot" => "request");
$requestAlias =& $_REQUEST["payload"]["slot"];
$requestCopy = $_REQUEST["payload"];

foreach ($requestCopy as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $_REQUEST["payload"]["slot"], "|", $requestCopy["slot"], "|", $_REQUEST["payload"]["plain"], "|", $requestCopy["plain"], "|";
$requestCopy["slot"] = "request-direct";
echo $value, "|";
$value = "request-tail";
echo $_REQUEST["payload"]["slot"], "|", $requestCopy["slot"];
