<?php
interface NamedPlugin {
    public function label();
    public function label_alias();
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
        PrimaryLabel::label as public label_alias;
    }
}

$plugin = new Plugin();
echo $plugin->label(), "|";
echo $plugin->label_alias(), "|";
echo $plugin->hooks(), "|";
echo method_exists($plugin, "label_alias") ? "alias-method" : "missing";
echo "|";
echo trait_exists("PrimaryLabel") ? "trait" : "missing";
