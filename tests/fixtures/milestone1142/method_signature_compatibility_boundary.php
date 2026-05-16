<?php
class Base {
    public function label($value) {
        return "base:" . $value;
    }
}

class Child extends Base {
    public function label($prefix, $value) {
        return $prefix . $value;
    }
}
