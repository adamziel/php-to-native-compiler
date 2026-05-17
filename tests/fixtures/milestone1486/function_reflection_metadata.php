<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract, TaggedContract {}

function &select_hook(HookContract|array|null $hook, HookContract&TaggedContract $tagged, $fallback = "seed"): HookContract|array|null {
    return $hook;
}

function raw_hook($value) {}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $type, $ending = "\n") {
    if ($type === null) {
        echo $label, "|null", $ending;
        return;
    }
    echo $label, "|", get_class($type), "|", yn($type instanceof ReflectionType), "|", yn($type->allowsNull()), $ending;
}

function param_line($label, $parameter) {
    $declaringClass = $parameter->getDeclaringClass();
    echo $label, "|", $parameter->getName(), "|", $parameter->getPosition(), "|", get_class($parameter->getDeclaringFunction()), "|", $parameter->getDeclaringFunction()->getName(), "|", yn($declaringClass === null), "|", yn($parameter->isDefaultValueAvailable()), "|", yn($parameter->hasType()), "\n";
}

$function = new ReflectionFunction("select_hook");
echo "fn|", $function->getName(), "|", get_class($function), "|", $function->getNumberOfParameters(), "|", $function->getNumberOfRequiredParameters(), "|", yn($function->hasReturnType()), "|", yn($function->returnsReference()), "\n";
line("return", $function->getReturnType());
foreach ($function->getParameters() as $index => $parameter) {
    param_line("param" . $index, $parameter);
}
param_line("direct", new ReflectionParameter("select_hook", "tagged"));
echo "raw|", yn((new ReflectionFunction("raw_hook"))->hasReturnType()), "|";
line("raw-return", (new ReflectionFunction("raw_hook"))->getReturnType(), "");
