<?php
class Box {
    public $store;

    public function __get($name) {
        $copy = $this->store[$name];
        $copy["plain"]["leaf"] = "copy";
        $copy["ref"]["leaf"] = "copy-ref";
        return $copy;
    }
}

$box = new Box();
$box->store = array(
    "item" => array(
        "plain" => array("leaf" => "source"),
        "ref" => array("leaf" => "ref-source"),
    ),
);
$alias =& $box->store["item"]["ref"]["leaf"];
$out = $box->item;

echo $box->store["item"]["plain"]["leaf"], "|";
echo $out["plain"]["leaf"], "|";
echo $alias, "|", $out["ref"]["leaf"];
