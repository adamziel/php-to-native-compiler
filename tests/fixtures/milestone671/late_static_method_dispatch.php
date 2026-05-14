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
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class;
    }

    public static function parentLabel() {
        return parent::label();
    }
}

echo Base::label(), "\n";
echo Child::label(), "\n";
echo Child::parentLabel();
