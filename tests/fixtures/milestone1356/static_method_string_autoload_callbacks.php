<?php
class StaticStringLoader {
    public static function load($name) {
        echo "string-static:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

spl_autoload_register("StaticStringLoader::load");

echo class_exists("StringLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("StringLoadedContract") ? "interface\n" : "missing-interface\n";

require_once __DIR__ . "/StringPlugin.inc";

$plugin = new StringPlugin();
echo $plugin->boot(), "\n";
echo trait_exists("StringLoadedTrait", false) ? "trait" : "missing-trait";
