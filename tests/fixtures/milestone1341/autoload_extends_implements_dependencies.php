<?php
function LoadDependency($name) {
    require_once __DIR__ . "/" . $name . ".inc";
}

spl_autoload_register("LoadDependency");

require_once __DIR__ . "/ChildPlugin.inc";

$plugin = new ChildPlugin("wp");
echo get_parent_class($plugin), "\n";
echo $plugin->name, ":", $plugin->label(), ":", $plugin->boot(), "\n";
echo is_a($plugin, "BaseContract") ? "base-contract\n" : "missing-base\n";
echo is_a($plugin, "LoadedContract") ? "loaded-contract" : "missing-loaded";
