<?php
class Milestone1776_CrossGotoBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->events[] = "offset-start";
        if ($offset === "target") {
            $this->events[] = "offset-if";
            goto selected_offset;
        }
        return $this->items["fallback"];
selected_offset:
        while (true) {
            $this->events[] = "offset-loop";
            goto after_offset_loop;
        }
after_offset_loop:
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1776_CrossGotoMagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        foreach (["unused", $name] as $candidate) {
            $this->events[] = "magic-foreach:" . $candidate;
            if ($candidate === $name) {
                goto selected_magic;
            }
        }
        return $this->store["fallback"];
selected_magic:
        switch ($name) {
            case "missing":
                $this->events[] = "magic-switch";
                goto return_magic;
        }
        return $this->store["fallback"];
return_magic:
        return $this->store[$name];
    }

    public function writeValue($name, $key, $value) {
        $this->store[$name][$key]["value"] = $value;
    }

    public function writePlain($name, $key, $value) {
        $this->store[$name][$key]["plain"]["value"] = $value;
    }

    public function readPlain($name, $key) {
        return $this->store[$name][$key]["plain"]["value"];
    }
}

$bagSource = "bag-seed";
$magicSource = "magic-seed";

$bagNode = ["value" => &$bagSource, "plain" => ["value" => "bag-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$bag = new Milestone1776_CrossGotoBag();
$box = new Milestone1776_CrossGotoMagicBox();

$bag->items["target"] = [];
$box->store["missing"] = [];

$bag["target"]["node"] = $bagNode;
$box->missing["node"] = $magicNode;

$bag["target"]["node"]["value"] = "bag-cross-goto";
$bag["target"]["node"]["plain"]["value"] = "bag-plain-cross-goto";
$box->writeValue("missing", "node", "magic-cross-goto");
$box->writePlain("missing", "node", "magic-plain-cross-goto");

echo $bagSource,
    "|",
    $bag->items["target"]["node"]["plain"]["value"],
    "|",
    $bag->events[0],
    "|",
    $bag->events[1],
    "|",
    $bag->events[2],
    "|",
    $magicSource,
    "|",
    $box->readPlain("missing", "node"),
    "|",
    $box->events[0],
    "|",
    $box->events[1],
    "|",
    $box->events[2];
