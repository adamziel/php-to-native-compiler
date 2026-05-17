<?php
class BaseBox {
    public $label;

    public function __construct($label) {
        $this->label = $label;
    }
}

class PluginBox extends BaseBox {
    public function __clone() {
        $this->label = $this->label . ":cloned";
    }
}

$source = new PluginBox("seed");
$copy = clone $source;
echo $source->label, "|", $copy->label;
