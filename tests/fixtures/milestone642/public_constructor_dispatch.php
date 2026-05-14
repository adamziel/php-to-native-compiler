<?php
class Box {
    public $name;
    public $count;

    public function __construct($name = "Ada") {
        $this->name = $name;
        $this->count = 1;
    }

    public function label() {
        return $this->name . ":" . $this->count;
    }
}

$box = new Box("Grace");
echo $box->label(), "\n";
$default = new Box();
echo $default->label();
