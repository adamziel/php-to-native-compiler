<?php
interface PluginContract {
    public function label_alias($hook);
}

trait PrimaryLabel {
    public function label($hook) {
        return "primary:" . $hook . ":" . get_class($this);
    }
}

trait FallbackLabel {
    public function label($hook) {
        return "fallback:" . $hook . ":" . get_class($this);
    }
}

trait HookTools {
    use PrimaryLabel, FallbackLabel {
        PrimaryLabel::label insteadof FallbackLabel;
        PrimaryLabel::label as public label_alias;
        PrimaryLabel::label as protected hidden_label;
    }

    public function boot($hook) {
        return $this->hidden_label($hook);
    }
}

class Plugin implements PluginContract {
    use HookTools;
}

$plugin = new Plugin();
echo $plugin->label("init"), "\n";
echo $plugin->label_alias("admin"), "\n";
echo $plugin->boot("rest"), "\n";
echo method_exists($plugin, "hidden_label") ? "hidden-exists\n" : "hidden-missing\n";

$methods = get_class_methods($plugin);
echo count($methods), "|";
echo in_array("label", $methods) ? "label" : "missing";
echo "|";
echo in_array("label_alias", $methods) ? "alias" : "missing";
echo "|";
echo in_array("boot", $methods) ? "boot\n" : "missing\n";

$hidden = new ReflectionMethod(Plugin::class, "hidden_label");
echo $hidden->getDeclaringClass()->getName(), "|";
echo $hidden->isProtected() ? "protected" : "not-protected";
