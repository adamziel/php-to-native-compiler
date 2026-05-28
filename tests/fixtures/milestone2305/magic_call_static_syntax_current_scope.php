<?php
class Milestone2305_MagicCallStaticBase {
    public function __call($name, $args) {
        echo "base:" . $name . ":" . implode(",", $args) . "\n";
    }

    public static function __callStatic($name, $args) {
        echo "base-static:" . $name . "\n";
    }
}

class Milestone2305_MagicCallStaticChild extends Milestone2305_MagicCallStaticBase {
    public $firstMagicCall = true;

    public function __call($name, $args) {
        if (!$this->firstMagicCall) {
            echo "\n";
        }
        $this->firstMagicCall = false;
        echo "child:" . $name . ":" . implode(",", $args);
    }

    public static function __callStatic($name, $args) {
        echo "child-static:" . $name . "\n";
    }

    public function test() {
        $class = "Milestone2305_MagicCallStaticBase";
        $object = $this;

        Milestone2305_MagicCallStaticBase::namedBase(1, "a");
        Milestone2305_MagicCallStaticChild::namedChild(2, "b");
        self::selfCall(3, "c");
        parent::parentCall(4, "d");
        static::lateCall(5, "e");
        $class::dynamicClass(6, "f");
        $object::dynamicObject(7, "g");
    }
}

$child = new Milestone2305_MagicCallStaticChild();
$child->test();
