<?php
namespace App\Core;

function label($name = "Ada") {
    return "hi " . $name;
}

$call = "App\\Core\\label";
echo $call("Ada"), "\n";

$local = "label";
echo function_exists($local) ? "yes" : "no";
