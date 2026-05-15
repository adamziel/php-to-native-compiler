<?php
$wp_filter = ["plugins_loaded" => ["count" => 1]];
$wp_actions = [];
function phpc_do_action($hook_name) {
    global $wp_filter, $wp_actions;
    $wp_actions[$hook_name] = true;
    echo $hook_name, ":", $wp_filter[$hook_name]["count"], "\n";
}
phpc_do_action("plugins_loaded");
echo isset($wp_actions["plugins_loaded"]) ? "recorded" : "missing";
