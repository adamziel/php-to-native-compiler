<?php
function &items_callback(&$items) {
    return $items;
}

class BaseBag {
    public static function &items(&$items) {
        return $items;
    }
}

class ChildBag extends BaseBag {
    public static function &items(&$items) {
        return $items;
    }

    public function runSelf(&$items) {
        foreach (self::items($items) as $key => &$item) {
            $item = "self:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }

    public function runParent(&$items) {
        foreach (parent::items($items) as $key => &$item) {
            $item = "parent:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }

    public function runStatic(&$items) {
        foreach (static::items($items) as $key => &$item) {
            $item = "static:" . $key;
            if ($key === "a") {
                $items["b"] = "bee";
            }
        }
        echo $items["a"], "|", $items["b"], "|", $item, "\n";
        unset($item);
    }
}

$bag = new ChildBag();

$items = ["a" => "aye"];
$bag->runSelf($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$bag->runParent($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$bag->runStatic($items);
$items["b"] = "direct";
echo $items["b"], "\n";

$items = ["a" => "aye"];
$class = "ChildBag";
foreach ($class::items($items) as $key => &$item) {
    $item = "class:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item, "\n";
unset($item);

$items = ["a" => "aye"];
foreach ($bag::items($items) as $key => &$item) {
    $item = "object:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item, "\n";
unset($item);

$items = ["a" => "aye"];
foreach (call_user_func_array("items_callback", array(&$items)) as $key => &$item) {
    $item = "callback:" . $key;
    if ($key === "a") {
        $items["b"] = "bee";
    }
}
echo $items["a"], "|", $items["b"], "|", $item;
