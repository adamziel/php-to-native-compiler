<?php
class Base {
    protected $shared;
    protected $count;

    public function seedBase($shared, $count) {
        $this->shared = $shared;
        $this->count = $count;
    }

    public function describeBase() {
        return $this->shared . ":" . $this->count;
    }
}

class Child extends Base {
    public function updateFromChild($other) {
        echo $this->shared, "\n";
        echo isset($other->shared) ? "peer-set\n" : "peer-unset\n";
        echo empty($other->shared) ? "peer-empty\n" : "peer-filled\n";
        $this->count += 2;
        ++$other->count;
        $other->shared ??= "filled";
        echo $this->describeBase(), "\n";
        echo $other->describeBase();
    }
}

$first = new Child();
$second = new Child();
$first->seedBase("first", 4);
$second->seedBase(null, 9);
$first->updateFromChild($second);
