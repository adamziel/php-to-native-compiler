<?php
class StaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

class ObjectLoader {
    public function load($name) {
        if ($name === "StaticLoadedBox") {
            return false;
        }
        echo "object:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

$loader = new ObjectLoader();
spl_autoload_register(array("StaticLoader", "load"));
spl_autoload_register(array($loader, "load"), true, true);

echo class_exists("StaticLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("ObjectLoadedContract") ? "interface\n" : "missing-interface\n";

require_once __DIR__ . "/ObjectPlugin.inc";

$plugin = new ObjectPlugin();
echo $plugin->boot(), "\n";
echo trait_exists("ObjectLoadedTrait", false) ? "trait" : "missing-trait";
