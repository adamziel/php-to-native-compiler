<?php
class Milestone1858_Box {
    public $trace = array();

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        for ($i = 0; $i < 2; $i = $i + 1) {
            $GLOBALS["trace"][] = "loop" . $i;
        }
        $slot = $name;
        return $GLOBALS["store"][$slot];
    }
}

$source = "seed";
$GLOBALS["store"] = array(
    "missing" => array("ref" => &$source, "plain" => array("value" => "copy")),
);
$GLOBALS["trace"] = array();

$box = new Milestone1858_Box();
$alias =& $box->missing;
$alias["ref"] = "changed";

$copy = $box->missing;
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $GLOBALS["store"]["missing"]["plain"]["value"], "|",
    implode(",", $box->trace), "|", implode(",", $GLOBALS["trace"]);
