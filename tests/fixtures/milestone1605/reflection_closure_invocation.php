<?php
$prefix = "wp";
$callback = function ($hook, $priority = 10) use ($prefix) {
    return $prefix . ":" . $hook . ":" . $priority;
};
$prefix = "changed";

$function = new ReflectionFunction($callback);
echo $function->invoke("init"), "\n";
echo $function->invokeArgs(array("save_post", 20)), "\n";
echo $function->getName(), "|", $function->getNumberOfParameters(), "/", $function->getNumberOfRequiredParameters();
