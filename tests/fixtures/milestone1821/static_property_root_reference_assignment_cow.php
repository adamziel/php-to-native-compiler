<?php
class Milestone1821Box {
    public static $slots = array("slot" => "s0");
}

$alias =& Milestone1821Box::$slots;
$alias["slot"] = "s1";
echo "slot=", Milestone1821Box::$slots["slot"], "|";

Milestone1821Box::$slots = array("slot" => "s2");
echo "alias=", $alias["slot"], "|";

$alias["slot"] = "s3";
echo "overwrite=", Milestone1821Box::$slots["slot"];
