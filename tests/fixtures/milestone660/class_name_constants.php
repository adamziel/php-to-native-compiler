<?php
class Box {}
class Root {}
class Base extends Root {
    public function baseNames() {
        return self::class . ":" . parent::class;
    }
}
class Child extends Base {
    public function childNames() {
        return self::class . ":" . parent::class;
    }

    public function inheritedNames() {
        return $this->baseNames();
    }
}

echo Box::class, "\n";
echo Box::CLASS, "\n";
echo Missing::class, "\n";
$child = new Child();
echo $child->childNames(), "\n";
echo $child->inheritedNames(), "\n";
echo "done";
