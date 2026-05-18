<?php
class Milestone1668_Nested_ArrayAccess_Bag implements ArrayAccess {
    private $storage;

    public function __construct($storage = []) {
        $this->storage = $storage;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->storage[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->storage[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->storage[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->storage[$offset]);
    }

    public function read($offset) {
        return $this->storage[$offset];
    }
}

$inner = new Milestone1668_Nested_ArrayAccess_Bag(["slot" => "seed", "return" => "pick"]);
$outer = new Milestone1668_Nested_ArrayAccess_Bag(["inner" => $inner]);

class Milestone1668_Magic_Box {
    public function &__get($name) {
        global $outer;
        return $outer;
    }
}

function milestone1668_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &milestone1668_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$box = new Milestone1668_Magic_Box();
milestone1668_mark($box->missing["inner"]["slot"], "direct");
echo $inner->read("slot"), "\n";

$args = array(&$box->missing["inner"]["slot"], "stored");
echo call_user_func_array("milestone1668_mark", $args), "|", $inner->read("slot"), "|", $args[0], "\n";

$returnArgs = array(&$box->missing["inner"]["return"], "return");
$alias =& call_user_func_array("milestone1668_pick", $returnArgs);
$alias = $alias . ":alias";
echo $inner->read("return"), "|", $returnArgs[0], "|", $alias;
