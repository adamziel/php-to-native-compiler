<?php
class Box {}

if (!trait_exists("Box")) {
    echo "class:not-trait\n";
}
if (!trait_exists("Missing")) {
    echo "missing:not-trait\n";
}
if (!trait_exists("Missing", false)) {
    echo "missing:false-autoload\n";
}

$call = "trait_exists";
if (!$call("Box", true)) {
    echo "dynamic:not-trait\n";
}
