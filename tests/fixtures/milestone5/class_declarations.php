<?php
class Box {
    public $value;
    private static $cache;

    protected function compute($input = "x") {
        return $input;
    }

    public static function make() {
        return "ok";
    }
}

class EmptyBox {}

echo "class metadata registered\n";
