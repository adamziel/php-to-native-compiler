<?php
interface RootContract {
    public function root($hook = "init");
}

interface HookContract extends RootContract {
    public static function register();
}

class BasePlugin {
    protected function inherited() {}
    private function baseHidden() {}
    public final function seal() {}
}

trait HookTools {
    public function helper() {}
    public function label() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public static function register() {}
    private function hidden() {}
    public function root($hook = "init") {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function dump_methods($label, $methods) {
    $lines = array();
    foreach ($methods as $method) {
        $lines[$method->getName()] = $method->getDeclaringClass()->getName() . ":" . $method->getModifiers() . ":" . yn($method->isStatic()) . yn($method->isAbstract());
    }
    foreach ($lines as $name => $line) {
        echo $label, "|", $name, "|", $line, "\n";
    }
}

function method_line($label, $method, $ending = "\n") {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), ":", $method->getModifiers(), ":", yn($method->isStatic()), yn($method->isAbstract()), $ending;
}

$plugin = new ReflectionClass(Plugin::class);
dump_methods("all", $plugin->getMethods());
dump_methods("public", $plugin->getMethods(ReflectionMethod::IS_PUBLIC));
dump_methods("static", $plugin->getMethods(ReflectionMethod::IS_STATIC));
echo "zero|", count($plugin->getMethods(0)), "\n";

$interface = new ReflectionClass(HookContract::class);
dump_methods("interface", $interface->getMethods());

$trait = new ReflectionClass(HookTools::class);
$traitMethods = $trait->getMethods();
method_line("trait", $traitMethods[0]);
method_line("trait", $traitMethods[1], "");
