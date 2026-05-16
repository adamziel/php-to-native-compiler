<?php
class Box {
    public $value;

    public function __construct($value = "default") {
        $this->value = $value;
    }
}

$class = "box";
$box = new $class("dynamic");
echo get_class($box), "|", $box->value;
