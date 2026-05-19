<?php
error_reporting(0);

class Milestone2080_Picker {
    public function &pick(&$value) {
        return $value["ref"];
    }
}

class Milestone2080_Box {
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

$picker = new Milestone2080_Picker();
$box = new Milestone2080_Box();
$leaf =& $box->leaf();

$alias =& call_user_func(array($picker, "pick"), $box->slot);
$alias["value"] = "copy";

echo $leaf["value"], "|", $box->plainValue();
