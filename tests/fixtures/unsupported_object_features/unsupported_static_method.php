<?php
class Box {
    public static function make() {}

    public function call() {
        self::make();
    }
}

$box = new Box();
$box->call();
