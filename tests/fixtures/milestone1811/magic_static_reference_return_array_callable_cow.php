<?php
class Milestone1811Holder {
    public $slots = array(
        "user" => "u0",
        "array" => "a0",
    );
}

$holder = new Milestone1811Holder();

class Milestone1811Magic {
    public static function &__callStatic($method, $args) {
        global $holder;
        echo "call:", $method, "|";
        return $holder->slots[$args[0]];
    }
}

$cb = array("Milestone1811Magic", "slot");
$alias =& call_user_func($cb, "user");
$alias = "u1";
echo "user=", $holder->slots["user"], "|";

$alias =& call_user_func_array($cb, array("array"));
$alias = "a1";
echo "array=", $holder->slots["array"];
