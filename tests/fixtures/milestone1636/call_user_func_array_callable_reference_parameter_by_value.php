<?php
function milestone1636_warning($errno, $errstr) {
    echo "warning:" . $errno . ":" . (str_contains($errstr, "must be passed by reference") ? "ref" : "other") . "\n";
    return true;
}

class Milestone1636_Filter {
    public $seen = "seed";

    public function mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        $this->seen = $this->seen . ":" . $suffix;
        return $value . ":" . $this->seen;
    }

    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

set_error_handler("milestone1636_warning", E_WARNING);
$filter = new Milestone1636_Filter();
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
echo call_user_func(array($filter, "mark"), $option, "object"), "|", $option, "|", $filter->seen, "\n";
echo call_user_func(array("Milestone1636_Filter", "tag"), $items["payload"]["slot"], "static"), "|", $items["payload"]["slot"];
