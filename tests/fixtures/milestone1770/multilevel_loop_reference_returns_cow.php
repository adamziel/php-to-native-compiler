<?php
class Milestone1770_BreakBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->events[] = "start";
        while (true) {
            while (true) {
                break 2;
            }
            $this->events[] = "unreached";
        }
        $this->events[] = "after-break";
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
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

class Milestone1770_ContinueBag implements ArrayAccess {
    public $items = [];
    public $events = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        for ($i = 0; $i < 2; $i++) {
            $this->events[] = $i;
            for ($j = 0; $j < 1; $j++) {
                if ($i === 0) {
                    continue 2;
                }
                if (!isset($this->items[$offset])) {
                    $this->items[$offset] = [];
                }
                return $this->items[$offset];
            }
        }
        return $this->items["fallback"];
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

class Milestone1770_MagicBox {
    public $store = [];
    public $events = [];

    public function &__get($name) {
        while (true) {
            try {
                while (true) {
                    break 2;
                }
            } finally {
                $this->events[] = "finally";
            }
        }
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        return $this->store[$name];
    }
}

$breakSource = "break-seed";
$continueSource = "continue-seed";
$magicSource = "magic-seed";

$breakNode = ["value" => &$breakSource, "plain" => ["value" => "break-copy"]];
$continueNode = ["value" => &$continueSource, "plain" => ["value" => "continue-copy"]];
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$breakBag = new Milestone1770_BreakBag();
$continueBag = new Milestone1770_ContinueBag();
$box = new Milestone1770_MagicBox();

$breakBag["break"]["node"] = $breakNode;
$continueBag["continue"]["node"] = $continueNode;
$box->missing["node"] = $magicNode;

$breakBag->items["break"]["node"]["value"] = "break-cow";
$breakBag->items["break"]["node"]["plain"]["value"] = "break-plain";
$continueBag->items["continue"]["node"]["value"] = "continue-cow";
$continueBag->items["continue"]["node"]["plain"]["value"] = "continue-plain";
$box->store["missing"]["node"]["value"] = "magic-cow";
$box->store["missing"]["node"]["plain"]["value"] = "magic-plain";

echo $breakSource,
    "|",
    $breakBag->items["break"]["node"]["plain"]["value"],
    "|",
    $breakBag->events[0],
    "|",
    $breakBag->events[1],
    "|",
    $continueSource,
    "|",
    $continueBag->items["continue"]["node"]["plain"]["value"],
    "|",
    $continueBag->events[0],
    "|",
    $continueBag->events[1],
    "|",
    $magicSource,
    "|",
    $box->store["missing"]["node"]["plain"]["value"],
    "|",
    $box->events[0];
