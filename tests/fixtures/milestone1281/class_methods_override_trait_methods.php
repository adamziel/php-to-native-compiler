<?php
interface HookContract {
    public function label($prefix);
    public function boot();
}

trait DefaultHooks {
    public function label($prefix, $fallback = null) {
        return $prefix . ":trait";
    }

    public function boot() {
        return "trait-boot";
    }
}

trait FallbackHooks {
    public function boot() {
        return "fallback-boot";
    }
}

class Plugin implements HookContract {
    use DefaultHooks, FallbackHooks;

    public function label($prefix) {
        return $prefix . ":class";
    }

    public function boot() {
        return "class-boot";
    }
}

$plugin = new Plugin();
echo $plugin->label("wp"), "|";
echo $plugin->boot(), "|";
echo method_exists($plugin, "label") ? "label-method" : "missing";
echo "|";
echo method_exists($plugin, "boot") ? "boot-method" : "missing";
