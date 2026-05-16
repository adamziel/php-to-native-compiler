<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString:$this->value\n";
        return "label:$this->value";
    }
}

class Holder {
    public $label;
}

$label = new Label();
$items = ["label" => $label];
$box = new Holder();
$box->label = $label;
echo "plain:$label\n";
echo "array:{$items['label']}\n";
echo "property:$box->label\n";
echo "chain:{$box->label}\n";
echo <<<TEXT
heredoc:$label
TEXT;
