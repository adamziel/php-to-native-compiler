<?php
function milestone1679_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1679_notice_handler", E_NOTICE);

class Milestone1679_Bag implements ArrayAccess {
    public $items = ["" => "empty"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return false; }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { }
}

class Milestone1679_RefBag implements ArrayAccess {
    public $items = ["" => "ref-empty"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Milestone1679_Holder {
    public $bag;
    public $dynamicBag;
    public $items = ["" => "plain-empty"];

    public function __construct($bag = null) {
        if ($bag !== null) {
            $this->bag = $bag;
            $this->dynamicBag = $bag;
        }
    }
}

class Milestone1679_Registry {
    public $holder;

    public function holder() {
        return $this->holder;
    }
}

function milestone1679_make_holder($bag) {
    return new Milestone1679_Holder($bag);
}

$bag = new Milestone1679_Bag();
$holders = ["box" => new Milestone1679_Holder($bag)];
$alias =& $holders["box"]->bag[];
$alias = "changed";
echo $alias, "|", $bag->items[""], "\n";

$property = "dynamicBag";
$dynamic =& $holders["box"]->{$property}[];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $bag->items[""], "\n";

$registry = new Milestone1679_Registry();
$registry->holder = new Milestone1679_Holder($bag);
$method =& $registry->holder()->bag[];
$method = "method-changed";
echo $method, "|", $bag->items[""], "\n";

$expr =& milestone1679_make_holder($bag)->bag[];
$expr = "expr-changed";
echo $expr, "|", $bag->items[""], "\n";

$refBag = new Milestone1679_RefBag();
$refHolders = ["box" => new Milestone1679_Holder($refBag)];
$ref =& $refHolders["box"]->bag[];
$ref = "ref-changed";
echo $ref, "|", $refBag->items[""], "\n";

$plainHolders = ["box" => new Milestone1679_Holder()];
$plain =& $plainHolders["box"]->items[];
$plain = "plain-changed";
echo $plain, "|", $plainHolders["box"]->items[0], "|", $plainHolders["box"]->items[""];
