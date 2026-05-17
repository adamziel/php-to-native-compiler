<?php
interface Hookable {
    public function register_hooks();
}

interface Labelable {
    public function label();
}

interface PluginContract extends Hookable, Labelable {
    public function boot();
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

trait HasLabel {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin implements PluginContract {
    use HasHooks, HasLabel {
        HasHooks::hooks as public register_hooks;
    }

    public function boot() {
        return "boot:" . get_class($this);
    }
}

$plugin = new Plugin();
echo $plugin instanceof Hookable ? "instanceof-hookable\n" : "missing-hookable\n";
echo $plugin instanceof Labelable ? "instanceof-labelable\n" : "missing-labelable\n";
echo is_a($plugin, "Hookable") ? "is-a-hookable\n" : "missing-is-a-hookable\n";
echo is_a($plugin, "Labelable") ? "is-a-labelable\n" : "missing-is-a-labelable\n";
echo is_subclass_of($plugin, "Hookable") ? "subclass-hookable\n" : "missing-subclass-hookable\n";
echo is_subclass_of($plugin, "Labelable") ? "subclass-labelable\n" : "missing-subclass-labelable\n";
echo $plugin->register_hooks(), "\n";
echo $plugin->label(), "\n";
echo $plugin->boot();
