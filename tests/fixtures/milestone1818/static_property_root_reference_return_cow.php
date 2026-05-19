<?php
class Milestone1818Base {
    public static $slots = array("base" => "b0");

    public static function &root() {
        return static::$slots;
    }

    public static function &dynamicRoot($class) {
        return $class::$slots;
    }
}

class Milestone1818Child extends Milestone1818Base {
    public static $slots = array("child" => "c0");
}

$alias =& Milestone1818Child::root();
$alias["child"] = "c1";
echo "child=", Milestone1818Child::$slots["child"], "|";

$class = "Milestone1818Base";
$alias =& Milestone1818Base::dynamicRoot($class);
$alias["base"] = "b1";
echo "base=", Milestone1818Base::$slots["base"], "|";

Milestone1818Base::$slots = array("base" => "b2");
echo "alias=", $alias["base"], "|";

$alias["base"] = "b3";
echo "overwrite=", Milestone1818Base::$slots["base"];
