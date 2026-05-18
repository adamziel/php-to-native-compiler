<?php
error_reporting(E_NOTICE);

function milestone1758_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1758_notice", E_NOTICE);

class Milestone1758_ByValueFalseMagicBox {
    public function __get($name) {
        return false;
    }
}

$box = new Milestone1758_ByValueFalseMagicBox();
$box->missing["leaf"] = "keyed";
$box->missing["outer"]["leaf"] = "nested";
$box->missing[] = "append";
$box->missing["outer"][] = "nested-append";

echo property_exists($box, "missing") ? "mutated" : "no-op";

error_reporting(0);

class Milestone1758_Box {
    public int $id = 1;
}

class Milestone1758_ByReferenceFalseMagicBox {
    public $store = false;

    public function &__get($name) {
        return $this->store;
    }
}

$typed = new Milestone1758_Box();
$alias =& $typed->id;
$referenceBox = new Milestone1758_ByReferenceFalseMagicBox();
$referenceBox->missing[] = array("copy" => &$alias);
$referenceBox->store[0]["copy"] = "2";

echo "\n", gettype($typed->id), ":", $typed->id, "|", gettype($referenceBox->store[0]["copy"]), ":", $referenceBox->store[0]["copy"];
