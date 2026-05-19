<?php
class Milestone1822Box {
    public static $slots = array(
        "slot" => "s0",
        "nested" => array("leaf" => "n0"),
    );
}

$alias =& Milestone1822Box::$slots["slot"];
$alias = "s1";
echo "slot=", Milestone1822Box::$slots["slot"], "|";

$leaf =& Milestone1822Box::$slots["nested"]["leaf"];
$leaf = "n1";
echo "nested=", Milestone1822Box::$slots["nested"]["leaf"];
