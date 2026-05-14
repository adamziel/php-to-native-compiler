<?php
class Base {
    public function inherited($other) {
        return "inherited:" . $other->seal();
    }

    protected function seal() {
        return "sealed";
    }
}

class Child extends Base {
    public function childCall($other) {
        return "child:" . $other->seal();
    }
}

$base = new Base();
$child = new Child();
echo get_parent_class($child), "\n";
echo get_parent_class("Child"), "\n";
echo is_subclass_of($child, "Base") ? "object-subclass" : "missing";
echo "\n";
echo is_subclass_of("Child", "Base") ? "string-subclass" : "missing";
echo "\n";
echo is_a($child, "Base") ? "object-is-a" : "missing";
echo "\n";
echo $child->inherited($base), "\n";
echo $child->childCall($child);
