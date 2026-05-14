<?php
class Base {
    public static function name() {
        return "base:" . static::class . ":" . get_called_class();
    }
}

class Child extends Base {
    public static function name() {
        return "child:" . static::class . ":" . get_called_class();
    }

    public static function parentName() {
        $class = Base::class;
        return $class::name();
    }
}

$base = "Base";
$child = "Child";
echo $base::name(), "\n";
echo $child::name(), "\n";
echo Child::parentName();
