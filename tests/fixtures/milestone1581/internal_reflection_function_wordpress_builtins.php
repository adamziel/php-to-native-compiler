<?php
$strpos = new ReflectionFunction("StrPos");
$returnType = $strpos->getReturnType();
$returnNames = array();
foreach ($returnType->getTypes() as $part) {
    $returnNames[] = $part->getName();
}
echo "strpos|", $strpos->getName(), "|", $strpos->getNumberOfParameters(), "/", $strpos->getNumberOfRequiredParameters(), "|", get_class($returnType), ":", implode("|", $returnNames), "|", $strpos->invoke("wp-admin", "admin"), "\n";
$offset = $strpos->getParameters()[2];
echo "offset|", $offset->getName(), "|", ($offset->isOptional() ? "1" : "0"), "|", ($offset->isDefaultValueAvailable() ? "1" : "0"), "|", $offset->getDefaultValue(), "|", $offset->getType()->getName(), "\n";

$substr = new ReflectionFunction("substr");
echo "substr|", $substr->invoke("save_post", 0, 4), "|", $substr->getNumberOfParameters(), "/", $substr->getNumberOfRequiredParameters(), "\n";
echo "trim|", (new ReflectionFunction("trim"))->invoke("  init  "), "|", (new ReflectionFunction("trim"))->getNumberOfParameters(), "/", (new ReflectionFunction("trim"))->getNumberOfRequiredParameters(), "\n";
echo "ltrim|", (new ReflectionFunction("ltrim"))->invoke("  admin"), "\n";
echo "rtrim|", (new ReflectionFunction("rtrim"))->invoke("hook  "), "\n";
$contains = new ReflectionFunction("str_contains");
echo "contains|", ($contains->invokeArgs(array("wp-admin/includes", "admin")) ? "1" : "0"), "|", $contains->getReturnType()->getName(), "\n";
echo "starts|", ((new ReflectionFunction("str_starts_with"))->invoke("rest_api_init", "rest") ? "1" : "0"), "\n";
echo "ends|", ((new ReflectionFunction("str_ends_with"))->invoke("template_redirect", "redirect") ? "1" : "0"), "\n";
echo "case|", (new ReflectionFunction("strcasecmp"))->invoke("REST", "rest"), "\n";
echo "path|", (new ReflectionFunction("basename"))->invoke("/var/www/wp-config.php", ".php"), "|", (new ReflectionFunction("dirname"))->invoke("/var/www/wp-content/plugins", 2), "\n";
echo "format|", (new ReflectionFunction("sprintf"))->invoke("hook:%s:%d", "init", 10), "\n";
$sprintfValues = (new ReflectionFunction("sprintf"))->getParameters()[1];
echo "variadic|", $sprintfValues->getName(), "|", ($sprintfValues->isVariadic() ? "1" : "0"), "|", ($sprintfValues->isOptional() ? "1" : "0"), "|", ($sprintfValues->isDefaultValueAvailable() ? "1" : "0"), "\n";
echo "implode|", (new ReflectionFunction("implode"))->invokeArgs(array("-", array("mu", "plugin"))), "\n";
echo "defined|", ((new ReflectionFunction("defined"))->invoke("PHP_VERSION") ? "1" : "0"), "\n";
echo "function|", ((new ReflectionFunction("function_exists"))->invoke("str_contains") ? "1" : "0"), "\n";
$sapi = new ReflectionFunction("php_sapi_name");
echo "sapi|", $sapi->getNumberOfParameters(), "/", $sapi->getNumberOfRequiredParameters(), "|", $sapi->invoke();
