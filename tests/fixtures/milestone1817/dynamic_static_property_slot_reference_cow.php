<?php
class Milestone1817Box {
    public static $slots = array(
        "named" => "n0",
        "dynamic" => "d0",
        "nested" => array("leaf" => "l0"),
    );

    public static function &pickNamed($key) {
        return Milestone1817Box::$slots[$key];
    }

    public static function &pickDynamic($class, $key) {
        return $class::$slots[$key];
    }

    public static function &pickDynamicNested($class, $key) {
        return $class::$slots["nested"][$key];
    }
}

$alias =& Milestone1817Box::pickNamed("named");
$alias = "n1";
echo "named=", Milestone1817Box::$slots["named"], "|";

$class = "Milestone1817Box";
$alias =& Milestone1817Box::pickDynamic($class, "dynamic");
$alias = "d1";
echo "dynamic=", Milestone1817Box::$slots["dynamic"], "|";

$alias =& Milestone1817Box::pickDynamicNested($class, "leaf");
$alias = "l1";
echo "nested=", Milestone1817Box::$slots["nested"]["leaf"];
