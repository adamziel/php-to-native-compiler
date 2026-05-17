<?php
interface StaticFactory {
    public static function make($name = "core");
}

interface PluginFactory extends StaticFactory {
    public static function boot($hook);
}

class Plugin implements PluginFactory {
    public static function make($name = "core") {
        return "make:" . $name;
    }

    public static function boot($hook) {
        return "boot:" . $hook;
    }
}

echo Plugin::make(), "|", Plugin::make("wp"), "|", Plugin::boot("init"), "|";
echo is_a("Plugin", "StaticFactory", true) ? "factory" : "missing";
