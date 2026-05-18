<?php
class HookProxy {
    public function __call($name, $args) {
        return $name . ":" . implode(",", $args);
    }
}

class ChildHookProxy extends HookProxy {}

class StaticHookProxy {
    public static function __callStatic($name, $args) {
        return $name . ":" . implode(",", $args);
    }
}

class StaticChildHookProxy extends StaticHookProxy {}

$proxy = new ChildHookProxy();

echo method_exists($proxy, "register") ? "object-method\n" : "object-missing\n";
echo is_callable(array($proxy, "register")) ? "object-callable\n" : "object-not-callable\n";
echo $proxy->register("init", 10), "\n";

echo method_exists("StaticChildHookProxy", "resolve") ? "static-method\n" : "static-missing\n";
echo is_callable(array("StaticChildHookProxy", "resolve")) ? "static-callable\n" : "static-not-callable\n";
echo StaticChildHookProxy::resolve("save_post", 20);
