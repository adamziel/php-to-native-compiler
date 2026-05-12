<?php
class Box {}

if (!enum_exists("Box")) {
    echo "class:not-enum\n";
}
if (!enum_exists("Missing")) {
    echo "missing:not-enum\n";
}
if (!enum_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}

$call = "enum_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-enum\n";
}
