<?php
class Box {
    public function open() {
        return "ok";
    }
    protected function seal() {}
    private static function cache() {}
    public static function named() {
        return "ok";
    }
}

$box = new Box();
$call = "is_callable";

echo is_callable(["Box", "open"]) ? "1" : "0";
echo is_callable(["BOX", "OPEN"]) ? "1" : "0";
echo is_callable([$box, "open"]) ? "1" : "0";
echo is_callable([$box, "seal"]) ? "1" : "0";
echo is_callable([$box, "cache"]) ? "1" : "0";
echo is_callable(["Box", "cache"]) ? "1" : "0";
echo is_callable(["Box", "named"]) ? "1" : "0";
echo is_callable([$box, "named"]) ? "1" : "0";
echo is_callable(["Missing", "open"]) ? "1" : "0";
echo is_callable(["Box", "missing"]) ? "1" : "0";
echo is_callable([1 => "Box", 2 => "open"]) ? "1" : "0";
echo is_callable(["Box", 42]) ? "1" : "0";
echo is_callable([42, "open"]) ? "1" : "0";
echo is_callable(["Box", "named"], false) ? "1" : "0";
echo $call([$box, "open"]) ? "1" : "0";
