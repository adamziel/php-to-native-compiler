<?php
class InvokeLoader {
    public function __invoke($name) {
        echo "invoke:", $name, "\n";
        require_once __DIR__ . "/" . $name . ".inc";
    }
}

$loader = new InvokeLoader();
spl_autoload_register($loader);

echo class_exists("InvokeLoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("InvokeLoadedContract") ? "interface\n" : "missing-interface\n";

require_once __DIR__ . "/InvokePlugin.inc";

$plugin = new InvokePlugin();
echo $plugin->boot(), "\n";
echo trait_exists("InvokeLoadedTrait", false) ? "trait" : "missing-trait";
