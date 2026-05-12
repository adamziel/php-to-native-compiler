<?php
class Box {}

if (class_exists("box")) {
    echo "box:exists\n";
}
if (class_exists("BOX", false)) {
    echo "box:false-autoload\n";
}
if (!class_exists("Missing")) {
    echo "missing:not-exists\n";
}

$call = "class_exists";
if ($call("Box", true)) {
    echo "dynamic:exists\n";
}
