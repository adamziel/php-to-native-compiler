<?php
error_reporting(0);

function &milestone2077_pick_ref(&$value) {
    return $value["ref"];
}

class Milestone2077_Box {
    private $store = array();

    public function __construct() {
        $this->store = array(
            "slot" => array(
                "ref" => array("value" => "original"),
                "plain" => array("value" => "plain-original"),
            ),
            "other" => array(
                "ref" => array("value" => "other"),
                "plain" => array("value" => "other-plain"),
            ),
        );
    }

    public function &leaf() {
        return $this->store["slot"]["ref"];
    }

    public function plainValue() {
        return $this->store["slot"]["plain"]["value"];
    }

    public function __get($name) {
        if ($name === "slot") {
            $bucket =& $this->store[$name];
        } else {
            $bucket =& $this->store["other"];
        }
        return $bucket;
    }
}

$box = new Milestone2077_Box();
$leaf =& $box->leaf();

$alias =& milestone2077_pick_ref($box->slot);
$alias["value"] = "copy";

echo $leaf["value"], "|", $box->plainValue();
