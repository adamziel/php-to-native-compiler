<?php
interface PluginContract extends Hookable {
    const CHILD = "contract";
    public function boot();
}

trait HasHooks {
    public function hook_impl() {
        return "hook:" . get_class($this);
    }
}

class Plugin implements PluginContract {
    use HasHooks {
        hook_impl as public register_hooks;
    }

    public function boot() {
        return self::PARENT . ":" . static::CHILD;
    }
}

interface Hookable {
    const PARENT = "base";
    public function register_hooks();
}

$plugin = new Plugin();
echo PluginContract::PARENT, "\n";
echo Plugin::PARENT, "\n";
echo $plugin instanceof Hookable ? "instanceof-base\n" : "missing-instanceof\n";
echo is_a("Plugin", "Hookable", true) ? "is-a-base\n" : "missing-is-a\n";
echo is_subclass_of("Plugin", "Hookable", true) ? "subclass-base\n" : "missing-subclass\n";
echo $plugin->boot(), "\n";
echo $plugin->register_hooks();
