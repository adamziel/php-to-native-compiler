<?php
trait HookLabels {
    public const NESTED = "nested";
    public function label($suffix) {
        return "nested:" . $suffix;
    }
}

trait HookTools {
    use HookLabels;
    public const SOURCE = "tools";
    public function register($hook) {
        return $this->label($hook) . ":" . self::SOURCE . ":" . self::NESTED;
    }
}

class Plugin {
    use HookTools;
}

$plugin = new Plugin();
echo $plugin->register("init"), "\n";
echo $plugin->label("admin"), "\n";
echo Plugin::SOURCE, "|", Plugin::NESTED, "\n";
$class = new ReflectionClass(Plugin::class);
echo implode(",", $class->getTraitNames()), "\n";
$names = array("register", "label");
$count = count($names);
foreach ($names as $index => $name) {
    $method = new ReflectionMethod(Plugin::class, $name);
    echo $name, "|", $method->getDeclaringClass()->getName(), $index + 1 === $count ? "" : "\n";
}
