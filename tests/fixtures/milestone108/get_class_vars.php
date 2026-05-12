<?php
class Box {
    public $name;
    protected $secret;
    private $token;
    public static $shared;
    private static $cache;
}

$vars = get_class_vars("BOX");
print_r($vars);
echo count($vars), "|", array_key_exists("name", $vars), "|", array_key_exists("shared", $vars), "\n";

$call = "get_class_vars";
$dynamic = $call("Box");
echo count($dynamic), "|", array_key_exists("secret", $dynamic);
