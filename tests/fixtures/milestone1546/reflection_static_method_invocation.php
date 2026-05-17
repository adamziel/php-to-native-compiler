<?php
class BaseHook {
    public static function attach($hook, $priority = 10) {
        return static::class . ":" . $hook . ":" . $priority;
    }
}

class ChildHook extends BaseHook {}

$base = new ReflectionMethod(BaseHook::class, "attach");
echo $base->invoke(null, "init"), "\n";
echo $base->invokeArgs(new BaseHook(), array("save_post", 20)), "\n";

$child = new ReflectionMethod(ChildHook::class, "attach");
echo $child->invoke(null, "plugins_loaded"), "\n";
echo $child->invokeArgs(new ChildHook(), array("shutdown", 50));
