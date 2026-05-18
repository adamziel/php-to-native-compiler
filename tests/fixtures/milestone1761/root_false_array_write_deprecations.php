<?php
function milestone1761_deprecated($errno, $message, $file, $line) {
    echo "deprecated:", $message, "\n";
    return true;
}

set_error_handler("milestone1761_deprecated", E_DEPRECATED);

class Milestone1761_Holder {
    public $items = false;
}

$direct = false;
$direct["leaf"] = "direct";

$globalRoot = false;
$GLOBALS["globalRoot"]["leaf"] = "global";

$holder = new Milestone1761_Holder();
$holder->items["leaf"] = "object";

$append = false;
$append[] = "append";

$reference = false;
$source = "source";
$reference["leaf"] =& $source;

echo $direct["leaf"], "|", $globalRoot["leaf"], "|", $holder->items["leaf"], "|", $append[0], "|", $reference["leaf"];
