<?php
class Box {
    public $store = array();

    public function &get() {
        $local =& $this->store;
        return $local;
    }
}

$box = new Box();
$box->store = array(
    "slot" => array(
        "ref" => array("leaf" => "source-ref"),
        "plain" => array("leaf" => "source-plain"),
    ),
);

$alias =& $box->store["slot"]["ref"];
$copy = $box->get();
$copy["slot"]["ref"]["leaf"] = "copy-ref";
$copy["slot"]["plain"]["leaf"] = "copy-plain";

echo $box->store["slot"]["ref"]["leaf"], "|",
    $alias["leaf"], "|",
    $box->store["slot"]["plain"]["leaf"], "|",
    $copy["slot"]["plain"]["leaf"];
