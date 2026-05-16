<?php
class Base {
    public static function label() {
        return "base";
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}

echo Child::label();
