<?php
class Box {
    public function open() {}
    protected function seal() {}
    private static function cache() {}
    public static function make() {}
}

$box = new box();
$object_methods = get_class_methods($box);
print_r($object_methods);
echo count($object_methods), "|", $object_methods[0], "|", $object_methods[1], "\n";

$class_methods = get_class_methods("BOX");
echo $class_methods[0], "|", $class_methods[1], "\n";

$call = "get_class_methods";
$dynamic = $call($box);
echo $dynamic[0], "|", $dynamic[1];
