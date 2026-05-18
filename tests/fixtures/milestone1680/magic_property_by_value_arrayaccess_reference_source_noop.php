<?php
function milestone1680_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1680_notice_handler", E_NOTICE);

class Milestone1680_ByValueBag implements ArrayAccess {
    public $items = ["name" => "seed", "outer" => ["slot" => "nested"], "" => "empty"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return false; }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { }
}

class Milestone1680_RefBag implements ArrayAccess {
    public $items = ["slot" => "ref-seed", "" => "ref-empty"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return isset($this->items[$offset]); }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) { return $this->items[$offset]; }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$valueBag = new Milestone1680_ByValueBag();
$refGetBag = new Milestone1680_ByValueBag();
$refBag = new Milestone1680_RefBag();
$plainStorage = ["slot" => "plain", "nested" => ["leaf" => "inside"]];

class Milestone1680_ByValueGetBox {
    public function __get($name) {
        global $valueBag;
        return $valueBag;
    }
}

class Milestone1680_ByReferenceGetBox {
    public function &__get($name) {
        global $refGetBag;
        return $refGetBag;
    }
}

class Milestone1680_RefArrayAccessBox {
    public function &__get($name) {
        global $refBag;
        return $refBag;
    }
}

class Milestone1680_PlainArrayBox {
    public function &__get($name) {
        global $plainStorage;
        return $plainStorage;
    }
}

$box = new Milestone1680_ByValueGetBox();
$key = "name";
$alias =& $box->missing[$key];
$alias = "changed";
echo "value-offset:", $alias, "|", $valueBag->items[$key], "\n";

$nested =& $box->missing["outer"]["slot"];
$nested = "nested-changed";
echo "value-nested:", $nested, "|", $valueBag->items["outer"]["slot"], "\n";

$property = "dynamicMissing";
$dynamic =& $box->{$property}[$key];
$dynamic = "dynamic-changed";
echo "value-dynamic:", $dynamic, "|", $valueBag->items[$key], "\n";

$append =& $box->missing[];
$append = "append-changed";
echo "value-append:", $append, "|", $valueBag->items[""], "\n";

$refGetBox = new Milestone1680_ByReferenceGetBox();
$refGetAlias =& $refGetBox->missing[$key];
$refGetAlias = "refget-changed";
echo "refget-offset:", $refGetAlias, "|", $refGetBag->items[$key], "\n";

$refGetNested =& $refGetBox->missing["outer"]["slot"];
$refGetNested = "refget-nested-changed";
echo "refget-nested:", $refGetNested, "|", $refGetBag->items["outer"]["slot"], "\n";

$refGetProperty = "dynamicMissing";
$refGetDynamic =& $refGetBox->{$refGetProperty}[$key];
$refGetDynamic = "refget-dynamic-changed";
echo "refget-dynamic:", $refGetDynamic, "|", $refGetBag->items[$key], "\n";

$refGetAppend =& $refGetBox->missing[];
$refGetAppend = "refget-append-changed";
echo "refget-append:", $refGetAppend, "|", $refGetBag->items[""], "\n";

$refBox = new Milestone1680_RefArrayAccessBox();
$refAlias =& $refBox->missing["slot"];
$refAlias = "ref-changed";
echo "ref-offset:", $refAlias, "|", $refBag->items["slot"], "\n";

$refAppend =& $refBox->missing[];
$refAppend = "ref-append-changed";
echo "ref-append:", $refAppend, "|", $refBag->items[""], "\n";

$plainBox = new Milestone1680_PlainArrayBox();
$plain =& $plainBox->missing["slot"];
$plain = "plain-changed";
echo "plain-offset:", $plain, "|", $plainStorage["slot"], "\n";

$plainNested =& $plainBox->missing["nested"]["leaf"];
$plainNested = "plain-nested-changed";
echo "plain-nested:", $plainNested, "|", $plainStorage["nested"]["leaf"], "\n";

$plainAppend =& $plainBox->missing[];
$plainAppend = "plain-append";
echo "plain-append:", $plainAppend, "|", $plainStorage[0];
