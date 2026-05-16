<?php
class Box {
    public static function &identity(&$value) {
        return $value;
    }

    public function run(&$value) {
        $alias =& self::identity($value);
        $alias = 2;
        echo "inside=", $value, "|";
    }
}

$box = new Box();
$value = 1;
$box->run($value);
echo "value=", $value, "|";

$value = 3;
$box->run($value);
echo "again=", $value;
