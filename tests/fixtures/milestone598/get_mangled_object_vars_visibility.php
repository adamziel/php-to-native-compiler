<?php
class Vault {
    public $label;
    protected $code;
    private $pin;
    public static $shared;
}

$vault = new Vault();
$vault->label = "safe";

$vars = get_mangled_object_vars($vault);
$keys = array_keys($vars);

echo count($vars), "\n";
echo strlen($keys[0]), "|", strlen($keys[1]), "|", strlen($keys[2]), "\n";
echo $keys[0] === "label", "|", $keys[1] === "code", "|", $keys[2] === "pin", "\n";
echo array_key_exists("code", $vars), "|", array_key_exists($keys[1], $vars), "\n";
echo $vars[$keys[0]], "|", $vars[$keys[1]] === null, "|", $vars[$keys[2]] === null, "\n";

$call = "get_mangled_object_vars";
$dynamic = $call($vault);
$dynamicKeys = array_keys($dynamic);
echo count($dynamic), "|", strlen($dynamicKeys[1]), "|", strlen($dynamicKeys[2]), "|", $dynamic[$dynamicKeys[0]];
