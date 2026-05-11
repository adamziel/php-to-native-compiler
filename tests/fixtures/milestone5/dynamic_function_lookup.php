<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}

$call = "greet";
echo $call("Ada"), "\n";

$upper = "GREET";
echo $upper("Lin", "."), "\n";

$length = "strlen";
echo $length("native"), "\n";

$counter = "count";
echo $counter(["a", "b"]), "\n";
