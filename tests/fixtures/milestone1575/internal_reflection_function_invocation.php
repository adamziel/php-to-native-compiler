<?php
$strlen = new ReflectionFunction("StrLen");
echo "name|", $strlen->getName(), "\n";
echo "file|", ($strlen->getFileName() ? "yes" : "no"), "\n";
echo "start|", ($strlen->getStartLine() ? "yes" : "no"), "\n";
echo "params|", $strlen->getNumberOfParameters() . "/" . $strlen->getNumberOfRequiredParameters(), "\n";
$params = $strlen->getParameters();
$param = $params[0];
echo "param|", $param->getName() . ":" . $param->getType()->getName(), "\n";
echo "return|", $strlen->getReturnType()->getName(), "\n";
echo "invoke|", $strlen->invoke("cache-key"), "\n";
echo "invokeArgs|", $strlen->invokeArgs(array("hook")), "\n";

$lower = new ReflectionFunction("strtolower");
echo "lower|", $lower->invoke("Save_Post"), "\n";
echo "lowerArgs|", $lower->invokeArgs(array("REST_API_INIT"));
