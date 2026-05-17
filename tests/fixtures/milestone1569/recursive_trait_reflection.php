<?php
trait BaseHooks {
    public function baseHook($suffix = "ok") {
        return "base:" . $suffix;
    }
}

trait HasHooks {
    use BaseHooks;

    public function directHook() {
        return "direct";
    }
}

class Plugin {
    use HasHooks;
}

function yn($value) {
    return $value ? "1" : "0";
}

$trait = new ReflectionClass(HasHooks::class);
echo "names|", implode(",", $trait->getTraitNames()), "\n";
foreach ($trait->getTraits() as $name => $reflected) {
    echo "trait|", $name, "|", $reflected->getName(), "|", yn($reflected->isTrait()), "\n";
}
foreach ($trait->getMethods() as $method) {
    echo "method|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", yn($method->isPublic()), yn($method->isAbstract()), "\n";
}
echo "has|", yn($trait->hasMethod("baseHook")), "\n";
$method = $trait->getMethod("baseHook");
echo "get|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "\n";
$constructed = new ReflectionMethod(HasHooks::class, "baseHook");
echo "construct|", $constructed->getName(), "|", $constructed->getDeclaringClass()->getName(), "\n";
$plugin = new Plugin();
echo "call|", $plugin->baseHook("wp"), "|", $plugin->directHook();
