<?php
class BaseBox {
    public static function &identity(&$value) {
        echo "base|";
        return $value;
    }

    public function run(&$value) {
        $alias =& static::identity($value);
        $alias = 2;
        echo "inside=", $value, "|";
    }
}

class Box extends BaseBox {
    public static function &identity(&$value) {
        echo "child|";
        return $value;
    }
}

$value = 1;
$base = new BaseBox();
$base->run($value);
echo "baseValue=", $value, "|";

$value = 3;
$box = new Box();
$box->run($value);
echo "childValue=", $value;
