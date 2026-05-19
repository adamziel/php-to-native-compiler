<?php
function milestone1923_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1923_notice", E_NOTICE);

class Milestone1923_Box {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

$function = "original";
$box = new Milestone1923_Box();
$box->store["missing"] = array(
    "id" => array("function" => &$function),
    "plain" => array("function" => "plain-original"),
);

$alias =& $box->missing["id"]["function"];
$alias = "alias";
$plain =& $box->missing["plain"]["function"];
$plain = "plain-alias";

$box->store["missing"]["id"]["function"] = "bucket";
$box->store["missing"]["plain"]["function"] = "plain-bucket";

echo $function, "|", $alias, "|", $box->store["missing"]["id"]["function"], "|",
    $plain, "|", $box->store["missing"]["plain"]["function"];
