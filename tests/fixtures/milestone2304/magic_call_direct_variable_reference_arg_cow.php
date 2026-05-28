<?php
class Forwarder {
    public function __call($name, $args) {
        $args[0]["shared"]["leaf"] = "changed-by-magic";
        $args[0]["plain"]["leaf"] = "copy-only";
        $args[1]["leaf"] = "second-copy-only";
    }
}

$shared = "original";
$items = array(
    "shared" => array("leaf" => "original"),
    "plain" => array("leaf" => "plain-original"),
);
$items["shared"]["leaf"] =& $shared;
$other = array("leaf" => "second-original");
(new Forwarder())->missing($items, $other);
echo $items["shared"]["leaf"] . "|" . $shared . "|" . $items["plain"]["leaf"] . "|" . $other["leaf"];
