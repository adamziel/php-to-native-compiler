<?php
class Milestone1810Holder {
    public $slots = array(
        "self" => "s0",
        "late" => "l0",
        "class" => "c0",
        "object" => "o0",
    );
}

$holder = new Milestone1810Holder();

class Milestone1810Base {
    public static function &__callStatic($method, $args) {
        global $holder;
        echo "call:", get_called_class(), ":", $method, "|";
        return $holder->slots[$args[0]];
    }

    public static function &throughSelf($key) {
        return self::slot($key);
    }

    public static function &throughStatic($key) {
        return static::slot($key);
    }
}

class Milestone1810Child extends Milestone1810Base {}

$alias =& Milestone1810Base::throughSelf("self");
$alias = "s1";
echo "self=", $holder->slots["self"], "|";

$alias =& Milestone1810Child::throughStatic("late");
$alias = "l1";
echo "late=", $holder->slots["late"], "|";

$class = "Milestone1810Base";
$alias =& $class::slot("class");
$alias = "c1";
echo "class=", $holder->slots["class"], "|";

$object = new Milestone1810Base();
$alias =& $object::slot("object");
$alias = "o1";
echo "object=", $holder->slots["object"];
