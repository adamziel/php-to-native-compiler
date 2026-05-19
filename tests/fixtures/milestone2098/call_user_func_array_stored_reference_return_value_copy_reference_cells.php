<?php
error_reporting(0);

function &milestone2098_pick(&$param) {
    return $param["ref"];
}

$ref = "original";
$payload = array("ref" => &$ref, "plain" => "plain-original");
$args = array($payload);

$alias =& call_user_func_array("milestone2098_pick", $args);
$alias = "copy";

echo $ref, "|", $payload["plain"];
