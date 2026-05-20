<?php
function lane2287_magic_passthrough($value) {
    return $value;
}

class Lane2287MagicRefBox {
    public $store = array();
    public $side = "";
    public $last = "";

    public function &__get($name) {
        $this->side = "side";
        $this->last = $name;
        $copy = lane2287_magic_passthrough($this->store);
        return $copy[$name];
    }
}

$box = new Lane2287MagicRefBox();
$box->store = array("slot" => array("child" => "source-child"));
$alias =& $box->store["slot"];

$box->slot["child"] = "copy-child";
$box->slot["extra"] = "alias-extra";

echo $box->store["slot"]["child"], "|", $alias["child"], "|";
echo isset($alias["extra"]) ? $alias["extra"] : "missing";
echo "|", $box->side, "|", $box->last;
