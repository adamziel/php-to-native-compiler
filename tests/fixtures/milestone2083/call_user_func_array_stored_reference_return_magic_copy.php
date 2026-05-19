<?php
error_reporting(0);

function &milestone2083_pick_ref(&$value) {
    return $value["ref"];
}

class Milestone2083_Box {
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

$box = new Milestone2083_Box();
$leaf =& $box->leaf();
$args = array($box->slot);

$alias =& call_user_func_array("milestone2083_pick_ref", $args);
$alias["value"] = "copy";

echo $leaf["value"], "|", $box->plainValue();
