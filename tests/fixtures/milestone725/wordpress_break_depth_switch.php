<?php
$wp_filter = ["plugins_loaded", "setup_theme"];
$done = false;
foreach ($wp_filter as $hook_name) {
    switch ($hook_name) {
        case "plugins_loaded":
            echo "found:", $hook_name, "\n";
            $done = true;
            break 2;
        default:
            echo "scan:", $hook_name, "\n";
    }
    echo "after-switch\n";
}
echo $done ? "done" : "missing";
