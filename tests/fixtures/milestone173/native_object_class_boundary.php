<?php
class Box {
    public $name;
    private $secret;
    public function open() {
        return "unused";
    }
}

$box = new Box();
$box->name = "Ada";

echo get_class($box), "\n";
echo is_object($box), "\n";
echo get_debug_type($box), "\n";
echo class_exists("box"), "\n";
echo property_exists($box, "name"), "\n";
echo method_exists("Box", "OPEN"), "\n";
print_r(get_class_methods($box));
print_r(get_class_vars("Box"));
print_r(get_object_vars($box));
echo "done";
