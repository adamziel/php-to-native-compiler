<?php
class Base {
    public $id;

    public function __construct($id = 7) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }
}

class Child extends Base {
    public $name;

    public function rename($name) {
        $this->name = $name;
    }
}

$child = new Child(11);
$child->rename("Ada");
echo $child->label(), "\n";
echo $child->id, "|", $child->name, "\n";
$default = new Child();
echo $default->label();
