<?php
interface NamedPlugin {
    public function label();
}

trait PrimaryLabel {
    public function label() {
        return "primary:" . get_class($this);
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback:" . get_class($this);
    }
}

trait HasHooks {
    public function hooks() {
        return "hooks:" . get_class($this);
    }
}

class Plugin implements NamedPlugin {
    use PrimaryLabel, FallbackLabel, HasHooks {
        PrimaryLabel::label insteadof FallbackLabel;
    }
}

$plugin = new Plugin();
echo $plugin->label(), "|";
echo $plugin->hooks(), "|";
echo method_exists($plugin, "label") ? "label-method" : "missing";
