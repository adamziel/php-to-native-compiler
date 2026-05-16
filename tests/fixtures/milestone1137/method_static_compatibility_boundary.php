<?php
class Base {
    public function label() {
        return "base";
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}
