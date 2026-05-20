<?php
class Lane2288MagicCopiedLocalBox {
    public $store = array();
    public $side = "";
    public $last = "";

    public function &__get($name) {
        $this->side = "side";
        $this->last = $name;
        $copy = $this->store[$name];
        return $copy;
    }
}

$box = new Lane2288MagicCopiedLocalBox();
$box->store = array(
    "slot" => array(
        "ref" => array("leaf" => "source-ref"),
        "plain" => array("leaf" => "source-plain"),
    ),
);
$alias =& $box->store["slot"]["ref"];

$box->slot["ref"]["leaf"] = "copy-ref";
$box->slot["plain"]["leaf"] = "copy-plain";

echo $box->store["slot"]["ref"]["leaf"], "|", $alias["leaf"], "|";
echo $box->store["slot"]["plain"]["leaf"];
echo "|", $box->side, "|", $box->last;
