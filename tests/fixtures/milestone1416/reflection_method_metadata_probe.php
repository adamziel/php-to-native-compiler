<?php
interface HookContract {
    public static function register();
    public function boot($hook = null);
}

abstract class BasePlugin {
    abstract protected function compute();
    public final function seal() {}
}

trait HookTools {
    public function helper() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public function __construct() {}
    public function boot($hook = null) {}
    public static function register() {}
    protected function compute() {}
    private function hidden() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $method) {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", $method->getModifiers(), "|", yn($method->isPublic()), yn($method->isProtected()), yn($method->isPrivate()), yn($method->isStatic()), yn($method->isFinal()), yn($method->isAbstract()), yn($method->isConstructor()), "\n";
}

echo ReflectionMethod::IS_PUBLIC, "|", ReflectionMethod::IS_PROTECTED, "|", ReflectionMethod::IS_PRIVATE, "|", ReflectionMethod::IS_STATIC, "|", ReflectionMethod::IS_FINAL, "|", ReflectionMethod::IS_ABSTRACT, "\n";
line("boot", new ReflectionMethod(Plugin::class, "boot"));
line("ctor", new ReflectionMethod(new Plugin(), "__construct"));
line("static", new ReflectionMethod(Plugin::class, "register"));
line("protected", new ReflectionMethod(Plugin::class, "compute"));
line("private", new ReflectionMethod(Plugin::class, "hidden"));
line("final", new ReflectionMethod(Plugin::class, "seal"));
line("abstract", new ReflectionMethod(BasePlugin::class, "compute"));
line("interface", new ReflectionMethod(HookContract::class, "register"));
$trait = new ReflectionMethod(HookTools::class, "helper");
echo "trait|", $trait->getName(), "|", $trait->getDeclaringClass()->getName(), "|", $trait->getModifiers(), "|", yn($trait->isPublic()), yn($trait->isProtected()), yn($trait->isPrivate()), yn($trait->isStatic()), yn($trait->isFinal()), yn($trait->isAbstract()), yn($trait->isConstructor());
