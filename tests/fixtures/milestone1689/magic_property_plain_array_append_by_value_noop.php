<?php
function milestone1689_notice_handler($errno, $message) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1689_notice_handler", E_NOTICE);

$storage = [];

class Milestone1689_ByValueMagicPlainArrayAppendBox {
    public function __get($name) {
        global $storage;
        return $storage;
    }
}

$box = new Milestone1689_ByValueMagicPlainArrayAppendBox();
$box->missing[] = ["by-value"];
echo array_key_exists(0, $storage) ? "mutated" : "no-op";
