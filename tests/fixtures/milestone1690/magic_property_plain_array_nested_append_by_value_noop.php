<?php
function milestone1690_notice_handler($errno, $message) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1690_notice_handler", E_NOTICE);

$storage = ["outer" => []];

class Milestone1690_ByValueMagicPlainArrayNestedAppendBox {
    public function __get($name) {
        global $storage;
        return $storage;
    }
}

$box = new Milestone1690_ByValueMagicPlainArrayNestedAppendBox();
$box->missing["outer"][] = ["by-value"];
echo array_key_exists(0, $storage["outer"]) ? "mutated" : "no-op";
