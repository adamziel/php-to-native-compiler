<?php
class Base {
    final public function label() {
        return "base";
    }
}

class Child extends Base {}

$child = new Child();
echo $child->label();
