<?php
class Milestone2304_MagicCallByrefArgs {
    public function __call($name, $args) {
        $args[0]["shared"]["leaf"] = "changed-by-magic";
        $args[0]["plain"]["leaf"] = "copy-only";
        $args[1]["leaf"] = "second-copy-only";
    }
}

$source = array(
    "shared" => array("leaf" => "original"),
    "plain" => array("leaf" => "plain-original"),
);
$leaf =& $source["shared"]["leaf"];
$second = array("leaf" => "second-original");

$object = new Milestone2304_MagicCallByrefArgs();
$object->missing($source, $second);

echo $source["shared"]["leaf"], "|", $leaf, "|";
echo $source["plain"]["leaf"], "|", $second["leaf"];
