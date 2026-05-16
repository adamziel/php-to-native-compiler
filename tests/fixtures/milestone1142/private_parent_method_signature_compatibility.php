<?php
class Base {
    private function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($prefix, $value) {
        return $prefix . $value;
    }
}

$child = new Child();
echo $child->label("child:", "ok");
