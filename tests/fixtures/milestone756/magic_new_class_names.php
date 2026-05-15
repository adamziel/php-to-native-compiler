<?php
class Base {
    public static function makeSelf() {
        return new self();
    }

    public static function makeStatic() {
        return new static();
    }
}

class Child extends Base {
    public function makeParent() {
        return new parent;
    }
}

echo get_class(Base::makeSelf()), "\n";
echo get_class(Child::makeSelf()), "\n";
echo get_class(Child::makeStatic()), "\n";
$child = new Child();
echo get_class($child->makeParent());
