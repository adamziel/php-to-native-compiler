<?php
interface HookDefaults {
    public const ACTION = "init";
    const PRIORITY = 10;
}

interface ChildDefaults extends HookDefaults {
    const GROUP = "plugins";
}

class Plugin implements ChildDefaults {
    public static function summary() {
        return self::ACTION . ":" . static::GROUP . ":" . static::PRIORITY;
    }
}

class OverridePlugin extends Plugin {
    public const PRIORITY = 20;
}

echo HookDefaults::ACTION, "\n";
echo ChildDefaults::ACTION, "\n";
echo Plugin::ACTION, "\n";
echo Plugin::GROUP, "\n";
echo Plugin::summary(), "\n";
echo OverridePlugin::summary(), "\n";
echo defined("ChildDefaults::PRIORITY") ? "defined\n" : "missing\n";
echo constant("Plugin::ACTION");
