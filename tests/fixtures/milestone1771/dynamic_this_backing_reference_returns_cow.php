<?php
class Milestone1771_DynamicPropertyBag implements ArrayAccess {
    private $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $property = "items";
        return isset($this->{$property}[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $property = "items";
        $suffix = "bucket";
        $this->events[] = $property;
        return $this->{$property}[$offset][$suffix];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $property = "items";
        $this->{$property}[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }

    public function writeValue($offset, $key, $value) {
        $property = "items";
        $this->{$property}[$offset]["bucket"][$key]["value"] = $value;
    }

    public function writePlain($offset, $key, $value) {
        $property = "items";
        $this->{$property}[$offset]["bucket"][$key]["plain"]["value"] = $value;
    }

    public function readPlain($offset, $key) {
        $property = "items";
        return $this->{$property}[$offset]["bucket"][$key]["plain"]["value"];
    }
}

class Milestone1771_DynamicMagicBox {
    private $missing = [];
    public $events = [];

    public function &__get($name) {
        $property = $name;
        $bucket = "bucket";
        $this->events[] = $property;
        return $this->{$property}[$bucket];
    }

    public function writeValue($name, $key, $value) {
        $property = $name;
        $this->{$property}["bucket"][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $property = $name;
        $this->{$property}["bucket"][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        $property = $name;
        return $this->{$property}["bucket"][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1771_DynamicPropertyBag();
$box = new Milestone1771_DynamicMagicBox();

$bag["outer"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag->writeValue("outer", "node", "bag-dynamic");
$bag->writePlain("outer", "node", "bag-plain-dynamic");
$box->writeValue("missing", "node", "magic-dynamic");
$box->writePlain("missing", "node", "magic-plain-dynamic");

echo $bagSource,
    "|",
    $bag->readPlain("outer", "node"),
    "|",
    $bag->events[0],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node"),
    "|",
    $box->events[0];
