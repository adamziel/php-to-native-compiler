<?php
$store = array(
    "slot" => "seed",
    "nested" => array("leaf" => "inner"),
    "dynamic" => "dyn",
);

class Milestone1662Box {
    public function &__get($name) {
        echo "get:$name\n";
        global $store;
        return $store;
    }
}

function milestone1662_touch(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$holders = array("box" => new Milestone1662Box());

milestone1662_touch($holders["box"]->missing["slot"], "param");
echo $store["slot"], "\n";

$alias =& $holders["box"]->missing["nested"]["leaf"];
$alias = $alias . ":alias";
echo $store["nested"]["leaf"], "|";
$store["nested"]["leaf"] = $store["nested"]["leaf"] . ":store";
echo $alias, "\n";

$property = "dynamicMissing";
milestone1662_touch($holders["box"]->{$property}["dynamic"], "dynamic");
echo $store["dynamic"];
