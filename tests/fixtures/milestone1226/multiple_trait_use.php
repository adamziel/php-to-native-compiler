<?php
interface Bootable {
    public function boot();
}

trait HasBoot {
    public function boot() {
        return "boot:" . get_class($this);
    }
}

trait HasLabel {
    public function label($value = "default") {
        return "label:" . $value;
    }
}

class Plugin implements Bootable {
    use HasBoot, HasLabel;
}

$plugin = new Plugin();
echo $plugin->boot(), "|";
echo $plugin->label("ok"), "|";
echo method_exists($plugin, "boot") ? "boot-method" : "missing";
echo "|";
echo trait_exists("HasLabel") ? "trait" : "missing";
