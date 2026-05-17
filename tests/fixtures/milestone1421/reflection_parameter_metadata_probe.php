<?php
class Plugin {
    public function boot(string $hook, &$value = "seed", $count = 3, ...$rest) {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function default_value($parameter) {
    if (!$parameter->isDefaultValueAvailable()) {
        return "-";
    }
    return $parameter->getDefaultValue();
}

function line($label, $parameter, $ending = "\n") {
    echo $label, "|", $parameter->getName(), "|", $parameter->getPosition(), "|", $parameter->getDeclaringClass()->getName(), "|", $parameter->getDeclaringFunction()->getName(), "|", yn($parameter->isOptional()), yn($parameter->isDefaultValueAvailable()), "|", default_value($parameter), "|", yn($parameter->isPassedByReference()), yn($parameter->isVariadic()), yn($parameter->hasType()), $ending;
}

$method = new ReflectionMethod(Plugin::class, "boot");
echo "counts|", $method->getNumberOfParameters(), "|", $method->getNumberOfRequiredParameters(), "\n";
foreach ($method->getParameters() as $index => $parameter) {
    line("param" . $index, $parameter);
}
line("named", new ReflectionParameter(array(Plugin::class, "boot"), "value"));
line("indexed", new ReflectionParameter(array(new Plugin(), "boot"), 2), "");
