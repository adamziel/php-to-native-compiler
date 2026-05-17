<?php
function ManualLoader($name) {
    echo "manual:", $name, "\n";
    require_once __DIR__ . "/" . $name . ".inc";
}

class ManualStaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

spl_autoload_register("ManualLoader");
spl_autoload_register(array("ManualStaticLoader", "load"));

$result = spl_autoload_call("ManualLoadedBox");
echo is_null($result) ? "null\n" : "not-null\n";
echo class_exists("ManualLoadedBox", false) ? "class\n" : "missing-class\n";

echo spl_autoload_unregister("ManualLoader") ? "unregistered\n" : "missing-loader\n";
spl_autoload_call("ManualLoadedContract");
echo interface_exists("ManualLoadedContract", false) ? "interface\n" : "missing-interface\n";

spl_autoload_call("ManualLoadedTrait");
echo trait_exists("ManualLoadedTrait", false) ? "trait" : "missing-trait";
