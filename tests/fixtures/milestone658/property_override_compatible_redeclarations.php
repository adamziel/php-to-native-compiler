<?php
class Base {
    private $privateValue;
}

class Child extends Base {
    public $privateValue;
}

$child = new Child();
$child->privateValue = "child-private";
print_r($child);
echo "done";
