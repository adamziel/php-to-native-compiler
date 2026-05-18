<?php
class Milestone1816Base {
    public static $slots = array(
        "base" => "b0",
        "nested" => array("base" => "bn0"),
    );

    public static function &pick($key) {
        echo "static:", get_called_class(), "|";
        return static::$slots[$key];
    }

    public static function &pickNested($key) {
        return static::$slots["nested"][$key];
    }
}

class Milestone1816Child extends Milestone1816Base {
    public static $slots = array(
        "child" => "c0",
        "nested" => array("child" => "cn0"),
    );
}

$alias =& Milestone1816Base::pick("base");
$alias = "b1";
echo "base=", Milestone1816Base::$slots["base"], "|";

$alias =& Milestone1816Child::pick("child");
$alias = "c1";
echo "child=", Milestone1816Child::$slots["child"], "|";

$alias =& Milestone1816Child::pickNested("child");
$alias = "cn1";
echo "nested=", Milestone1816Child::$slots["nested"]["child"];
