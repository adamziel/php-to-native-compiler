<?php
class Milestone1823Base {
    public static $slots = array("base" => "b0", "object" => "o0");

    public static function mutate($class, $object) {
        $root =& static::$slots;
        $root["base"] = "b1";

        $slot =& $class::$slots["base"];
        $slot = "b2";

        $objectRoot =& $object::$slots;
        $objectRoot["object"] = "o1";
    }
}

Milestone1823Base::mutate("Milestone1823Base", new Milestone1823Base());
echo "base=", Milestone1823Base::$slots["base"], "|";
echo "object=", Milestone1823Base::$slots["object"];
