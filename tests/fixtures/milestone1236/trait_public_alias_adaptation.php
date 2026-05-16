<?php
interface Registrable {
    public function register_hooks();
}

trait HasHooks {
    public function hooks($suffix = "default") {
        return "hooks:" . $suffix . ":" . get_class($this);
    }
}

trait HasLabel {
    public function label() {
        return "label:" . get_class($this);
    }
}

class Plugin implements Registrable {
    use HasHooks, HasLabel {
        HasHooks::hooks as public register_hooks;
    }
}

$plugin = new Plugin();
echo $plugin->hooks("direct"), "|";
echo $plugin->register_hooks("alias"), "|";
echo $plugin->label(), "|";
echo method_exists($plugin, "register_hooks") ? "alias-method" : "missing";
echo "|";
echo trait_exists("HasHooks") ? "trait" : "missing";
