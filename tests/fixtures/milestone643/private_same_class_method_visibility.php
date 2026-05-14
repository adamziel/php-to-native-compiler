<?php
class Box {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function label() {
        return $this->prefix() . ":" . $this->name;
    }

    public function labelOther($other) {
        return $other->prefix() . ":" . $other->name;
    }

    private function prefix() {
        return "private";
    }
}

$left = new Box("Ada");
$right = new Box("Grace");
echo $left->label(), "\n";
echo $left->labelOther($right);
