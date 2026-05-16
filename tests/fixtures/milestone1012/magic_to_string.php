<?php
class Label {
    public $value = "core";

    public function __toString() {
        echo "toString\n";
        return "label:" . $this->value;
    }
}

$label = new Label();
echo $label, "\n";
echo (string) $label, "\n";
echo "prefix-" . $label, "\n";
print $label;
