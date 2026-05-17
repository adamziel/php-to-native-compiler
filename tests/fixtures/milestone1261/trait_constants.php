<?php
trait HookDefaults {
    public const ACTION = "init";
    const PRIORITY = 10;

    public function hook_key() {
        return self::ACTION . ":" . static::PRIORITY;
    }
}

class Plugin {
    use HookDefaults;

    public static function action() {
        return self::ACTION;
    }
}

class ChildPlugin extends Plugin {
    public const PRIORITY = 20;
}

echo Plugin::ACTION, "\n";
echo Plugin::PRIORITY, "\n";
echo Plugin::action(), "\n";

$plugin = new Plugin();
echo $plugin->hook_key(), "\n";

$child = new ChildPlugin();
echo $child->hook_key();
