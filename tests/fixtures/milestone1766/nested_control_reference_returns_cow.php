<?php
class Milestone1766_BranchBag implements ArrayAccess {
    public $left = [];
    public $right = [];
    public $hits = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->left[$offset]) || isset($this->right[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        if ($offset === "left") {
            if (!isset($this->left[$offset])) {
                $this->left[$offset] = [];
            }
            return $this->left[$offset];
        }
        if (!isset($this->right[$offset])) {
            $this->right[$offset] = [];
        }
        return $this->right[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->right[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->left[$offset], $this->right[$offset]);
    }
}

class Milestone1766_MagicBox {
    public $left = [];
    public $right = [];
    public $hits = [];

    public function &__get($name) {
        $this->hits[] = $name;
        if ($name === "magicLeft") {
            if (!isset($this->left[$name])) {
                $this->left[$name] = [];
            }
            return $this->left[$name];
        }
        if (!isset($this->right[$name])) {
            $this->right[$name] = [];
        }
        return $this->right[$name];
    }

    public function read($side, $key, $field) {
        if ($side === "left") {
            return $this->left["magicLeft"][$key][$field]["value"];
        }
        return $this->right["magicRight"][$key][$field]["value"];
    }
}

$leftSource = "left-seed";
$rightSource = "right-seed";
$magicSource = "magic-seed";

$leftNode = ["value" => &$leftSource, "plain" => ["value" => "left-copy"]];
$rightNode = ["value" => &$rightSource, "plain" => ["value" => "right-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1766_BranchBag();
$bag["left"]["node"] = $leftNode;
$bag["right"]["node"] = $rightNode;
$bag->left["left"]["node"]["value"] = "left-changed";
$bag->right["right"]["node"]["value"] = "right-changed";
$bag->left["left"]["node"]["plain"]["value"] = "left-plain";
$bag->right["right"]["node"]["plain"]["value"] = "right-plain";

$box = new Milestone1766_MagicBox();
$box->magicLeft["node"] = $magicNode;
$box->magicLeft["node"]["value"] = "magic-changed";
$box->magicLeft["node"]["plain"]["value"] = "magic-plain";

echo $leftSource,
    "|",
    $rightSource,
    "|",
    $magicSource,
    "|",
    $bag->left["left"]["node"]["plain"]["value"],
    "|",
    $bag->right["right"]["node"]["plain"]["value"],
    "|",
    $box->read("left", "node", "plain"),
    "|",
    $bag->hits[0],
    "|",
    $bag->hits[1],
    "|",
    $box->hits[0],
    "|",
    $box->hits[1];
