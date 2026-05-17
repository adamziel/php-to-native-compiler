<?php
class Plugin {
    public function boot(string $hook, ?int $count, Plugin $plugin = null, array $items = null, $raw = null) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $parameter, $ending = "\n") {
    $type = $parameter->getType();
    if ($type === null) {
        echo $label, "|null|", yn($parameter->allowsNull()), $ending;
        return;
    }
    echo $label, "|", get_class($type), "|", $type->getName(), "|", yn($type->allowsNull()), yn($parameter->allowsNull()), yn($type->isBuiltin()), yn($type instanceof ReflectionType), $ending;
}

$method = new ReflectionMethod(Plugin::class, "boot");
foreach ($method->getParameters() as $parameter) {
    line($parameter->getName(), $parameter);
}
line("direct", new ReflectionParameter(array(new Plugin(), "boot"), "count"), "");
