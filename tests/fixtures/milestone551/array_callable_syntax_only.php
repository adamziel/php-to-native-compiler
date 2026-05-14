<?php
class Box {
    public function open() {
        return "ok";
    }
}

$box = new Box();
$call = "is_callable";

echo is_callable(["Box", "open"], true) ? "1" : "0";
echo is_callable([$box, "open"], true) ? "1" : "0";
echo is_callable(["Missing", "open"], true) ? "1" : "0";
echo is_callable(["Box", "not valid"], true) ? "1" : "0";
echo is_callable([1 => "Box", 2 => "open"], true) ? "1" : "0";
echo is_callable(["class" => "Box", "method" => "open"], true) ? "1" : "0";
echo is_callable(["Box"], true) ? "1" : "0";
echo is_callable(["Box", 42], true) ? "1" : "0";
echo is_callable([42, "open"], true) ? "1" : "0";
echo is_callable($box, true) ? "1" : "0";
echo $call([$box, "open"], true) ? "1" : "0";
