<?php
class Base {
    public $id;

    public function baseLabel() {
        return "base:" . $this->id;
    }

    protected function bumpBase($amount) {
        $this->id = $this->id + $amount;
    }
}

class Child extends Base {
    public function __construct($id) {
        $this->id = $id;
    }

    private function suffix() {
        return "child";
    }

    public function label() {
        return self::baseLabel() . ":" . self::suffix();
    }

    public function bump($amount) {
        self::bumpBase($amount);
    }
}

$child = new Child(3);
echo $child->label(), "\n";
$child->bump(5);
echo $child->label(), "\n";

class Ancestor {
    public function label() {
        return "ancestor";
    }

    public function callSelf() {
        return self::label();
    }
}

class Descendant extends Ancestor {
    public function label() {
        return "descendant";
    }
}

$descendant = new Descendant();
echo $descendant->callSelf();
