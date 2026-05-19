<?php
error_reporting(0);

$source = "seed";

function milestone1834_make_array() {
    global $source;
    return array(
        "ref" => &$source,
        "plain" => array("leaf" => "v"),
    );
}

$alias =& milestone1834_make_array()["ref"];
$alias = "changed";
$plain =& milestone1834_make_array()["plain"]["leaf"];
$plain = "detached";

echo $source, "|", $alias, "|", $plain;
