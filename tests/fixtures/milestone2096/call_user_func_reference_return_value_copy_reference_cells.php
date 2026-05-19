<?php
error_reporting(0);

function &milestone2096_pick(&$param) {
    return $param["ref"];
}

$ref = "original";
$payload = array("ref" => &$ref, "plain" => "plain-original");

$alias =& call_user_func("milestone2096_pick", $payload);
$alias = "copy";

echo $ref, "|", $payload["plain"];
