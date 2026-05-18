<?php
class Milestone1815Magic {
    public static $slots = array(
        "slot" => "start",
        "nested" => array("leaf" => "n0"),
    );

    public static function &__callStatic($method, $args) {
        echo "call:", $method, "|";
        if ($method === "nested") {
            return self::$slots["nested"][$args[0]];
        }
        return self::$slots[$args[0]];
    }
}

$alias =& Milestone1815Magic::slot("slot");
$alias = "changed";
echo "slot=", Milestone1815Magic::$slots["slot"], "|";

$alias =& Milestone1815Magic::nested("leaf");
$alias = "n1";
echo "nested=", Milestone1815Magic::$slots["nested"]["leaf"];
