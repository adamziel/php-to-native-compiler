<?php
class Router {
    public static function known() {
        return "known";
    }

    public static function throughSelf() {
        return self::route("self", 1);
    }

    public static function throughStatic() {
        return static::route("late", 2);
    }

    public static function __callStatic($method, $args) {
        echo "static:$method\n";
        return $method . ":" . $args[0] . ":" . $args[1] . ":" . get_called_class();
    }
}

class Child extends Router {}

$class = "Router";
$object = new Router();
echo Router::known(), "\n";
echo Router::route("posts", 7), "\n";
echo Router::throughSelf(), "\n";
echo Child::throughStatic(), "\n";
echo $class::route("class", 3), "\n";
echo $object::route("object", 4);
