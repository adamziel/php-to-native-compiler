<?php
trait PrimaryLabel {
    public function label() {
        return "primary";
    }
}

trait FallbackLabel {
    public function label() {
        return "fallback";
    }
}

class Plugin {
    use PrimaryLabel, FallbackLabel;
}

$plugin = new Plugin();
echo $plugin->label(), "\n";
