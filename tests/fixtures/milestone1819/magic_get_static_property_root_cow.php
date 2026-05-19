<?php
class Milestone1819Magic {
    public static $slots = array(
        "slot" => "s0",
        "nested" => array("leaf" => "n0"),
    );

    public function &__get($name) {
        echo "get:", $name, "|";
        return self::$slots;
    }
}

$box = new Milestone1819Magic();
$alias =& $box->missing;
$alias["slot"] = "s1";
echo "slot=", Milestone1819Magic::$slots["slot"], "|";

$box->another["nested"]["leaf"] = "n1";
echo "nested=", Milestone1819Magic::$slots["nested"]["leaf"];
