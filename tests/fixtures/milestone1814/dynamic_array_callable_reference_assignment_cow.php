<?php
class Milestone1814Holder {
    public $slots = array("magic" => "m0");
}

class Milestone1814Picker {
    public function &pick(&$items, $key) {
        echo "object:", $key, "|";
        return $items[$key];
    }
}

class Milestone1814Magic {
    public static function &__callStatic($method, $args) {
        global $holder;
        echo "magic:", $method, "|";
        return $holder->slots[$args[0]];
    }
}

$items = array("object" => "o0");
$picker = new Milestone1814Picker();
$cb = array($picker, "pick");
$alias =& $cb($items, "object");
$alias = "o1";
echo "object=", $items["object"], "|";

$holder = new Milestone1814Holder();
$cb = array("Milestone1814Magic", "slot");
$alias =& $cb("magic");
$alias = "m1";
echo "magic=", $holder->slots["magic"];
