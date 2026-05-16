<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString:$this->value\n";
        return "label:" . $this->value;
    }
}

class Box {
    public $text = "box:";
}

$label = new Label();
$text = "prefix:";
$text .= $label;
echo $text, "\n";
$value = $label;
$value .= ":tail";
echo $value, "\n";
$box = new Box();
$box->text .= $label;
echo $box->text;
