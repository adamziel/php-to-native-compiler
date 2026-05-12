<?php
class Box {}

if (!interface_exists("Box")) {
    echo "class:not-interface\n";
}
if (!interface_exists("Missing")) {
    echo "missing:not-interface\n";
}
if (!interface_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}

$call = "interface_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-interface\n";
}
