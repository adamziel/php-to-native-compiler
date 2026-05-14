<?php
class Base {
    public $id;

    public function __construct($id = 7) {
        $this->id = $id;
    }

    public function label() {
        return "base:" . $this->id;
    }

    protected function bumpBase($amount) {
        $this->id = $this->id + $amount;
    }
}

class Child extends Base {
    public $name;

    public function __construct($id, $name) {
        parent::__construct($id);
        $this->name = $name;
    }

    public function label() {
        return parent::label() . ":" . $this->name;
    }

    public function bump($amount) {
        parent::bumpBase($amount);
    }
}

$child = new Child(4, "Ada");
echo $child->label(), "\n";
$child->bump(5);
echo $child->label();
