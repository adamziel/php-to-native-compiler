<?php
class Base {
    public $base = "base";
    public static $counter = 1;
}

class Plugin extends Base {
    public string $name = "hook";
    public array $log = array("start");
}

function label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if ($value === null) {
        return "null";
    }
    return $value;
}

$plugin = new Plugin();
$name = new ReflectionProperty(Plugin::class, "name");
echo "name|get|", $name->getValue($plugin), "|", $plugin->name, "\n";
$name->setValue($plugin, "save");
echo "name|set|", $name->getValue($plugin), "|", $plugin->name, "\n";
$name->setValue($plugin, 123);
echo "name|coerce|", gettype($plugin->name), ":", $plugin->name, "\n";

$base = new ReflectionProperty(Plugin::class, "base");
$base->setValue($plugin, "inherited");
echo "base|", $base->getDeclaringClass()->getName(), "|", $base->getValue($plugin), "|", $plugin->base, "\n";

$log = new ReflectionProperty(Plugin::class, "log");
$log->setValue($plugin, array("first", "second"));
echo "log|", label($log->getValue($plugin)), "|", count($plugin->log), "\n";

$static = new ReflectionProperty(Base::class, "counter");
echo "static|get|", $static->getValue(), "|", Base::$counter, "\n";
$static->setValue(41);
echo "static|set1|", $static->getValue(null), "|", Base::$counter, "\n";
$static->setValue($plugin, 42);
echo "static|set2|", $static->getValue($plugin), "|", Base::$counter;
