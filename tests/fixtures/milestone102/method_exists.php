<?php
class Box {
    public function open() {}
    protected function seal() {}
    private static function cache() {}
}

$box = new box();
if (method_exists($box, "open")) {
    echo "object:open\n";
}
if (method_exists($box, "SEAL")) {
    echo "object:seal\n";
}
if (method_exists($box, "cache")) {
    echo "object:static\n";
}
if (method_exists("BOX", "CACHE")) {
    echo "class:static\n";
}
if (!method_exists("Box", "missing")) {
    echo "class:missing\n";
}
if (!method_exists("Missing", "open")) {
    echo "missing-class:false\n";
}

$call = "method_exists";
if ($call($box, "OPEN")) {
    echo "dynamic:exists\n";
}
