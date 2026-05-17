<?php
trait HasOptions {
    public $options = array("autoload" => "yes");
    protected $label = "wp-hook";
}

trait HasSameOptions {
    public $options = array("autoload" => "yes");
}

class Plugin {
    use HasOptions, HasSameOptions;

    public function label() {
        return $this->label;
    }
}

$plugin = new Plugin();
echo $plugin->options["autoload"], "\n";
$plugin->options["autoload"] = "no";
echo $plugin->options["autoload"], "\n";
echo $plugin->label(), "\n";

$class = new ReflectionClass("Plugin");
echo $class->hasProperty("options") ? "has-options\n" : "missing-options\n";
echo $class->hasProperty("label") ? "has-label\n" : "missing-label\n";
$property = $class->getProperty("options");
echo $property->getDeclaringClass()->getName(), "\n";
echo $property->hasDefaultValue() ? "default\n" : "no-default\n";
$default = $property->getDefaultValue();
echo $default["autoload"];
