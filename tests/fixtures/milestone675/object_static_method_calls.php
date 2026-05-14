<?php
class Base {
    protected static function hidden() {
        return "hidden:" . static::class;
    }

    public static function name() {
        return "base:" . static::class;
    }

    public static function label() {
        return static::name() . ":" . static::hidden();
    }

    public function fromThis() {
        return $this::label();
    }
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class;
    }

    public static function callOn($object) {
        return $object::label();
    }
}

class Vault {
    private static function key() {
        return "key:" . static::class;
    }

    public function reveal($other) {
        return $other::key();
    }
}

$base = new Base();
$child = new Child();
$vault = new Vault();
echo $base::label(), "\n";
echo $child::label(), "\n";
echo $child->fromThis(), "\n";
echo Child::callOn($child), "\n";
echo $vault->reveal(new Vault());
