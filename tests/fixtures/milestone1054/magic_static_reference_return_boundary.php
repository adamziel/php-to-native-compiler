<?php
class Holder {
    public $slots = array("slot" => "start");
}

$holder = new Holder();

class Box {
    public static function &__callStatic($method, $args) {
        global $holder;
        echo "call:", $method, "|";
        return $holder->slots[$args[0]];
    }
}

$alias =& Box::missing("slot");
$alias = "changed";
echo $holder->slots["slot"];
