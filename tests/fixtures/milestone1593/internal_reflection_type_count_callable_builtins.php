<?php
function type_name($parameter) {
    $type = $parameter->getType();
    if ($type === null) {
        return "";
    }
    if ($type instanceof ReflectionUnionType) {
        $names = array();
        foreach ($type->getTypes() as $part) {
            $names[] = $part->getName();
        }
        return implode("|", $names);
    }
    return $type->getName();
}

function param_line($label, $parameter) {
    echo $label, "|", $parameter->getName(), "|", type_name($parameter), "|", ($parameter->isOptional() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? "1" : "0"), "|", ($parameter->isPassedByReference() ? "1" : "0");
    if ($parameter->isDefaultValueAvailable()) {
        $default = $parameter->getDefaultValue();
        echo "|", $default === null ? "null" : ($default ? "true" : "false");
    }
    echo "\n";
}

$isArray = new ReflectionFunction("is_array");
echo "is_array|", $isArray->getNumberOfParameters(), "/", $isArray->getNumberOfRequiredParameters(), "|", $isArray->getReturnType()->getName(), "|", ($isArray->invoke(array("hook")) ? "1" : "0"), "|", ($isArray->invoke("hook") ? "1" : "0"), "\n";
param_line("is_array:param0", $isArray->getParameters()[0]);

$isObject = new ReflectionFunction("is_object");
echo "is_object|", ($isObject->invoke(new stdClass()) ? "1" : "0"), "|", ($isObject->invoke(array()) ? "1" : "0"), "\n";

$isString = new ReflectionFunction("is_string");
echo "is_string|", ($isString->invoke("save_post") ? "1" : "0"), "|", ($isString->invoke(42) ? "1" : "0"), "\n";

$isScalar = new ReflectionFunction("is_scalar");
echo "is_scalar|", ($isScalar->invoke("save_post") ? "1" : "0"), "|", ($isScalar->invoke(array()) ? "1" : "0"), "\n";

$count = new ReflectionFunction("count");
echo "count|", $count->getNumberOfParameters(), "/", $count->getNumberOfRequiredParameters(), "|", $count->getReturnType()->getName(), "|", $count->invoke(array("a", "b")), "\n";
param_line("count:param0", $count->getParameters()[0]);
param_line("count:param1", $count->getParameters()[1]);

$exists = new ReflectionFunction("array_key_exists");
echo "exists|", $exists->getNumberOfParameters(), "/", $exists->getNumberOfRequiredParameters(), "|", ($exists->invoke("hook", array("hook" => "init")) ? "1" : "0"), "|", ($exists->invoke("missing", array("hook" => "init")) ? "1" : "0"), "\n";
param_line("exists:param0", $exists->getParameters()[0]);
param_line("exists:param1", $exists->getParameters()[1]);

$callable = new ReflectionFunction("is_callable");
echo "callable|", $callable->getNumberOfParameters(), "/", $callable->getNumberOfRequiredParameters(), "|", ($callable->invoke("strlen") ? "1" : "0"), "|", ($callable->invoke("missing_function") ? "1" : "0"), "|", ($callable->invoke("Class::method", true) ? "1" : "0"), "\n";
param_line("callable:param1", $callable->getParameters()[1]);
$callableName = $callable->getParameters()[2];
echo "callable:param2|", $callableName->getName(), "|", type_name($callableName), "|", ($callableName->isOptional() ? "1" : "0"), "|", ($callableName->isDefaultValueAvailable() ? "1" : "0"), "|", ($callableName->isPassedByReference() ? "1" : "0"), "|", ($callableName->getDefaultValue() === null ? "null" : "value");
