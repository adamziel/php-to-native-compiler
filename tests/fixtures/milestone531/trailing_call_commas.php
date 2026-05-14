<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}

echo greet("Ada",), "\n";
echo greet("Lin", ".",), "\n";
echo strlen("native",), "\n";

$call = "greet";
echo $call("Grace",), "\n";
$length = "strlen";
echo $length("compiler",);
