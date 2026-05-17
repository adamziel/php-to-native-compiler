<?php
interface HookContract {
    public function register($hook);
}

class HookPlugin implements HookContract {
    public function register($hook) {
        return "plugin:" . $hook;
    }
}

$concrete = new ReflectionMethod(HookPlugin::class, "register");
echo $concrete->invoke(new HookPlugin(), "init"), "\n";

$interface = new ReflectionMethod(HookContract::class, "register");
echo $interface->invoke(new HookPlugin(), "save_post");
