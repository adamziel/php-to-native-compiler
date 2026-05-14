<?php
class Counter {
    public static $count;

    public static function bump($step = 1) {
        self::$count ??= 0;
        self::$count += $step;
        return self::$count;
    }
}
class Base {
    public static function name() {
        return self::class;
    }
}
class Child extends Base {}
class Hidden {
    private static $secret;

    public static function setSecret($value) {
        self::$secret = $value;
        return self::$secret;
    }
}

echo Counter::bump(), "\n";
echo Counter::bump(4), "\n";
echo Child::name(), "\n";
echo Hidden::setSecret("ok");
