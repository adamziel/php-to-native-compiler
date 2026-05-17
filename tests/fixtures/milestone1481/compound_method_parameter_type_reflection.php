<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract {}
class TaggedHook extends Hook implements TaggedContract {}
class OtherHook {}

class Plugin {
    public function select(HookContract|OtherHook|null $hook, HookContract&TaggedContract $tagged): HookContract|OtherHook|null {}
    public function tagged(): HookContract&TaggedContract {}
    public function raw($value) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function type_names($type) {
    $names = array();
    foreach ($type->getTypes() as $inner) {
        $names[] = $inner->getName() . ":" . yn($inner->isBuiltin()) . ":" . yn($inner->allowsNull());
    }
    return implode(",", $names);
}

function line($label, $type, $ending = "\n") {
    if ($type === null) {
        echo $label, "|null", $ending;
        return;
    }
    echo $label, "|", get_class($type), "|", yn($type instanceof ReflectionType), "|", yn($type->allowsNull()), "|", type_names($type), $ending;
}

$method = new ReflectionMethod(Plugin::class, "select");
$params = $method->getParameters();
echo "method|", yn($method->hasReturnType()), "|", $method->getNumberOfParameters(), "\n";
line("return-union", $method->getReturnType());
line("param-union", $params[0]->getType());
line("param-intersection", $params[1]->getType());
line("return-intersection", (new ReflectionMethod(Plugin::class, "tagged"))->getReturnType());
echo "raw|", yn((new ReflectionMethod(Plugin::class, "raw"))->hasReturnType()), "|";
line("raw-return", (new ReflectionMethod(Plugin::class, "raw"))->getReturnType(), "");
