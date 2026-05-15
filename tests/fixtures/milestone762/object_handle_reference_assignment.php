<?php
class Box {
    public $name;
}

function remember($key) {
    global $items;
    static $box = null;
    if ($box === null) {
        $box = new Box();
    }
    $box->name = "stored";
    $items[$key] =& $box;
    return $items[$key];
}

$value = remember("primary");
echo $value->name;
