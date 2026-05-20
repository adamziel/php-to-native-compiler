<?php
function lane2287_wrap_literal($value) {
    return array("wrap" => array($value));
}

$source = array(
    "bucket" => array("leaf" => "source"),
    "plain" => array("leaf" => "plain-source"),
);
$bucket =& $source["bucket"];
$wrapped = lane2287_wrap_literal($source);

$wrapped["wrap"][0]["bucket"]["leaf"] = "literal-ref";
$wrapped["wrap"][0]["plain"]["leaf"] = "literal-plain";

echo $source["bucket"]["leaf"], "|", $bucket["leaf"], "|";
echo $wrapped["wrap"][0]["bucket"]["leaf"], "|";
echo $source["plain"]["leaf"], "|", $wrapped["wrap"][0]["plain"]["leaf"];
