<?php
class Base {
    public $id;
    protected $secret;
    private $token;
    public static $shared;
}

class Child extends Base {
    public $name;
}

$child = new Child();
$child->id = 7;
$child->name = "Ada";
$vars = get_object_vars($child);
$classVars = get_class_vars("Child");
echo $child->id, "|", $child->name, "\n";
echo count($vars), "|", $vars["id"], "|", $vars["name"], "|", array_key_exists("secret", $vars), "\n";
echo count($classVars), "|", array_key_exists("name", $classVars), "|", array_key_exists("id", $classVars), "|", array_key_exists("shared", $classVars), "\n";
echo property_exists($child, "id") ? "prop:id" : "missing";
echo "\n";
echo property_exists($child, "secret") ? "prop:secret" : "missing";
echo "\n";
echo property_exists($child, "token") ? "bad" : "private-parent:false";
