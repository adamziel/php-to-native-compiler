<?php
class Milestone1809Holder {
    public $slots = array("slot" => "start");
}

$holder = new Milestone1809Holder();

class Milestone1809Magic {
    public static function &__callStatic($method, $args) {
        global $holder;
        echo "static:", $method, "|";
        return $holder->slots[$args[0]];
    }
}

$alias =& Milestone1809Magic::slot("slot");
$alias = "changed";
echo "magic-static|", $holder->slots["slot"];
