<?php
class Box {
    public static function make($value) {
        return "made:" . $value;
    }
}

echo Box::make("ok");
