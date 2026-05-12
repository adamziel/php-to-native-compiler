<?php
class Box {
    public $name;
    public $count;
}

$box = new box();
$box->name = "Ada";
$box->count = 3;

$vars = get_mangled_object_vars($box);
print_r($vars);
echo count($vars), "|", $vars["name"], "|", $vars["count"], "\n";

$call = "get_mangled_object_vars";
$dynamic = $call($box);
echo count($dynamic), "|", $dynamic["name"];
