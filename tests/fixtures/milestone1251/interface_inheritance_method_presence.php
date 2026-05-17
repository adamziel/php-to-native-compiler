<?php
interface Hookable {
    public function register_hooks();
}

interface PluginContract extends Hookable {
    public function label();
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
}

$plugin = new Plugin();
echo $plugin instanceof Hookable ? "instanceof-parent\n" : "missing-parent\n";
echo is_a($plugin, "Hookable") ? "is-a-parent\n" : "missing-is-a\n";
echo is_subclass_of($plugin, "Hookable") ? "subclass-parent\n" : "missing-subclass\n";
echo $plugin->register_hooks(), "\n";
echo $plugin->label();
