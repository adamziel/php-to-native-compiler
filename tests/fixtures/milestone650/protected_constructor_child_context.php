<?php
class Base {
    public $id;

    protected function __construct($id = 5) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }
}

class Child extends Base {
    public function __construct() {}

    public function makeBase($id) {
        return new Base($id);
    }

    public function makeDefaultBase() {
        return new Base();
    }
}

$child = new Child();
$base = $child->makeBase(12);
echo $base->label(), "\n";
$default = $child->makeDefaultBase();
echo $default->label();
