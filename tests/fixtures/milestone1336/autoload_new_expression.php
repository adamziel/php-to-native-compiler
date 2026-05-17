<?php
function LoadClass($class) {
    echo "load:", $class, "\n";
    require_once __DIR__ . "/" . $class . ".inc";
}

spl_autoload_register("LoadClass");

$box = new LoadedBox();
echo $box->name, "\n";

$class = "DynamicBox";
$dynamic = new $class("dynamic");
echo $dynamic->name;
