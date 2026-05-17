<?php
function LoadObjectDependency($name) {
    require_once __DIR__ . "/" . $name . ".inc";
}

spl_autoload_register("LoadObjectDependency");

echo trait_exists("AutoloadedProbe", false) ? "false-loaded\n" : "false-skip\n";
echo trait_exists("AutoloadedProbe") ? "probe-loaded\n" : "probe-missing\n";

require_once __DIR__ . "/Plugin.inc";

$plugin = new Plugin();
echo $plugin->boot(), "\n";
echo trait_exists("LoadedHook", false) ? "hook-loaded" : "hook-missing";
