<?php
interface RootContract {
    public function root();
}

interface HookContract extends RootContract {
    public function boot($hook = null);
}

trait HookTools {
    public function helper() {
        return "helper";
    }
}

class BasePlugin {
    public function root() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public function boot($hook = null) {}
}

$plugin = new Plugin();
$class = new ReflectionClass($plugin);
echo $class->getName(), "\n";
echo $class->getShortName(), "\n";
echo $class->isInstantiable() ? "instantiable\n" : "not-instantiable\n";
echo $class->hasMethod("boot") ? "boot-method\n" : "missing-boot\n";
echo $class->hasMethod("helper") ? "helper-method\n" : "missing-helper\n";
print_r($class->getInterfaceNames());

$parent = $class->getParentClass();
echo $parent ? $parent->getName() . "\n" : "no-parent\n";

$interface = new ReflectionClass(HookContract::class);
echo $interface->isInterface() ? "interface\n" : "not-interface\n";
echo $interface->hasMethod("root") ? "root-method\n" : "missing-root\n";
print_r($interface->getInterfaceNames());

$trait = new ReflectionClass(HookTools::class);
echo $trait->isTrait() ? "trait\n" : "not-trait\n";
echo $trait->hasMethod("helper") ? "trait-helper" : "missing-helper";
