<?php
class Compat_SplFixedArray {
    private $internalArray;

    public function __construct() {
        $this->internalArray = ["name" => "Ada"];
    }

    public function toArray() {
        return (array) $this->internalArray;
    }
}

$box = new Compat_SplFixedArray();
$items = $box->toArray();
echo $items["name"];
