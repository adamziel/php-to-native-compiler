<?php
class Base {
    public function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($value, $suffix = "!") {
        return "child:" . $value . $suffix;
    }
}

$child = new Child();
echo $child->label("one"), "\n";
echo $child->label("two", "?");
