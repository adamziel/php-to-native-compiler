<?php
class Base {
    public static $count;

    public static function bump($step = 1) {
        self::$count ??= 0;
        self::$count += $step;
        return self::$count;
    }

    protected static function prefix() {
        return self::class;
    }

    public static function label() {
        return self::prefix();
    }
}

class Child extends Base {
    public static function parentBump($step) {
        return parent::bump($step);
    }

    public static function parentPrefix() {
        return parent::prefix();
    }
}

echo Base::bump(), "\n";
echo Child::parentBump(4), "\n";
echo Base::label(), "\n";
echo Child::parentPrefix();
