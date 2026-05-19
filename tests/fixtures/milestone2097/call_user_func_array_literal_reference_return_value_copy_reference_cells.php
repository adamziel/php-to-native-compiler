<?php
error_reporting(0);

function &milestone2097_pick(&$param) {
    return $param["ref"];
}

$ref = "original";
$payload = array("ref" => &$ref, "plain" => "plain-original");

$alias =& call_user_func_array("milestone2097_pick", array($payload));
$alias = "copy";

echo $ref, "|", $payload["plain"];
