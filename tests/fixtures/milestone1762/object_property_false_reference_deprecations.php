<?php
function milestone1762_deprecated($errno, $message, $file, $line) {
    echo "deprecated:", $message, "\n";
    return true;
}

set_error_handler("milestone1762_deprecated", E_DEPRECATED);

class Milestone1762_Holder {
    public $items = false;
    public $appendItems = false;
}

$holder = new Milestone1762_Holder();
$source = "source";

$holder->items["leaf"] =& $source;
$holder->appendItems[] =& $source;

echo $holder->items["leaf"], "|", $holder->appendItems[0];
