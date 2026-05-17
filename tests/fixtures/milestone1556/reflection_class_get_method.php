<?php
interface HookContract {
    public static function register();
}

class BasePlugin {
    protected function inherited() {}
}

trait HookTools {
    public function helper() {}
}

class Plugin extends BasePlugin implements HookContract {
    use HookTools;

    public static function register() {}
    private function hidden() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $method, $end = "\n") {
    echo $label, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "|", $method->getModifiers(), "|", yn($method->isPublic()), yn($method->isProtected()), yn($method->isPrivate()), yn($method->isStatic()), $end;
}

$plugin = new ReflectionClass(Plugin::class);
line("static", $plugin->getMethod("register"));
line("private", $plugin->getMethod("hidden"));
line("inherited", $plugin->getMethod("inherited"));
line("trait-composed", $plugin->getMethod("helper"));

$contract = new ReflectionClass(HookContract::class);
line("interface", $contract->getMethod("register"));

$trait = new ReflectionClass(HookTools::class);
line("trait", $trait->getMethod("helper"), "");
