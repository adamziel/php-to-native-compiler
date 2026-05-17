<?php
function Loader($class) {
    echo "load:", $class, "\n";
    require_once __DIR__ . "/autoloaded_metadata.inc";
}

spl_autoload_register("Loader");

echo class_exists("MissingBox", false) ? "false-loaded\n" : "false-skip\n";
echo class_exists("LoadedBox") ? "class\n" : "missing-class\n";
echo interface_exists("LoadedContract") ? "interface" : "missing-interface";
