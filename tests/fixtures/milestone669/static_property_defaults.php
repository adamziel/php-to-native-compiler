<?php
class Defaults {
    public static $name = "Ada";
    public static $count = 2 + 3;
    protected static $secret = "ok";

    public static function read() {
        return self::$name . ":" . self::$count . ":" . self::$secret;
    }
}

echo Defaults::$name, "\n";
echo Defaults::$count, "\n";
Defaults::$count += 4;
echo Defaults::read();
