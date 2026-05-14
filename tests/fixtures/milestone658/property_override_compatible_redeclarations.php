<?php
class Base {
    private $privateValue;
    protected $shared;
    public $name;

    public function describeBase() {
        return $this->shared . ":" . $this->name;
    }
}

class Child extends Base {
    public $privateValue;
    public $shared;
    public $name;
}

$child = new Child();
$child->privateValue = "child-private";
$child->shared = "child-shared";
$child->name = "child-name";
echo $child->describeBase(), "\n";
print_r($child);
echo "done";
