<?php
$_REQUEST["payload"] = array("slot" => "start");
$payload =& $_REQUEST["payload"];
$callback = function ($suffix) use (&$payload) {
    $payload["slot"] = $payload["slot"] . ":" . $suffix;
    return $payload["slot"];
};

echo $callback("direct"), "|", $_REQUEST["payload"]["slot"], "\n";
echo call_user_func($callback, "call"), "|", $_REQUEST["payload"]["slot"], "\n";
echo call_user_func_array($callback, array("array")), "|", $_REQUEST["payload"]["slot"], "\n";
$reflected = new ReflectionFunction($callback);
echo $reflected->invoke("reflect"), "|", $_REQUEST["payload"]["slot"], "\n";

class Milestone1642_Box {
    public $items = array("slot" => "box");
}

$box = new Milestone1642_Box();
$item =& $box->items["slot"];
$propertyCallback = function ($suffix) use (&$item) {
    $item = $item . ":" . $suffix;
    return $item;
};

echo $propertyCallback("property"), "|", $box->items["slot"], "\n";

