<?php
$callback = function (string $hook, $priority = 10): string { return $hook; };
$function = new ReflectionFunction($callback);
$suffix = "tests/fixtures/milestone1587/closure_reflection_metadata.php";
echo "fn|", $function->getName(), "|", get_class($function), "|", $function->getNumberOfParameters(), "/", $function->getNumberOfRequiredParameters(), "|", ($function->returnsReference() ? "1" : "0"), "|", ($function->hasReturnType() ? "1" : "0"), "|", $function->getReturnType()->getName(), "\n";
echo "source|", substr($function->getFileName(), -strlen($suffix)), "|", $function->getStartLine(), "|", $function->getEndLine(), "|", ($function->getDocComment() === false ? "1" : "0"), "\n";
foreach ($function->getParameters() as $index => $parameter) {
    $type = $parameter->getType();
    $declaring = $parameter->getDeclaringFunction();
    echo "param", $index, "|", $parameter->getName(), "|", ($parameter->isOptional() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? "1" : "0"), "|", ($parameter->isDefaultValueAvailable() ? $parameter->getDefaultValue() : ""), "|", ($type ? $type->getName() : ""), "|", $declaring->getName(), "\n";
}

$arrow = fn($value): int => 42;
$arrowReflection = new ReflectionFunction($arrow);
echo "arrow|", $arrowReflection->getName(), "|", $arrowReflection->getNumberOfParameters(), "/", $arrowReflection->getNumberOfRequiredParameters(), "|", $arrowReflection->getReturnType()->getName(), "|", $arrowReflection->getStartLine(), "|", $arrowReflection->getEndLine();
