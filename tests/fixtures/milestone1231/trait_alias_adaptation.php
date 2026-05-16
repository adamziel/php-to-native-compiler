<?php
interface Registrable {
    public function register_hooks();
}

trait HasHooks {
    public function hooks($suffix = "default") {
        return "hooks:" . $suffix . ":" . get_class($this);
    }
}

class Plugin implements Registrable {
    use HasHooks {
        hooks as register_hooks;
    }
}

$plugin = new Plugin();
echo $plugin->hooks("direct"), "|";
echo $plugin->register_hooks("alias"), "|";
echo method_exists($plugin, "register_hooks") ? "alias-method" : "missing";
echo "|";
echo trait_exists("HasHooks") ? "trait" : "missing";
