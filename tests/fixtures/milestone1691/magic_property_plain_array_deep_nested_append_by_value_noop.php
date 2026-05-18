<?php
function milestone1691_notice_handler($errno, $message) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1691_notice_handler", E_NOTICE);

$storage = ["outer" => ["inner" => []]];

class Milestone1691_ByValueMagicPlainArrayDeepNestedAppendBox {
    public function __get($name) {
        global $storage;
        return $storage;
    }
}

$box = new Milestone1691_ByValueMagicPlainArrayDeepNestedAppendBox();
$box->missing["outer"]["inner"][] = ["by-value"];
echo array_key_exists(0, $storage["outer"]["inner"]) ? "mutated" : "no-op";
