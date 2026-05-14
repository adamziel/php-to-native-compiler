<?php
class Base {
    public const NAME = "base";
    protected const SECRET = "secret";

    public static function describe() {
        return static::NAME . ":" . static::class . ":" . static::SECRET;
    }

    public function instanceDescribe() {
        return static::NAME . ":" . static::class;
    }
}

class Child extends Base {
    public const NAME = "child";

    public static function parentDescribe() {
        return parent::describe();
    }
}

echo Base::describe(), "\n";
echo Child::describe(), "\n";
echo Child::parentDescribe(), "\n";
$child = new Child();
echo $child->instanceDescribe(), "\n";
