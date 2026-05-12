<?php
class Box {}
class Crate {}

$box = new box();
if (!is_subclass_of($box, "Box")) {
    echo "object:exact-false\n";
}
if (!is_subclass_of($box, "Crate")) {
    echo "object:other-false\n";
}
if (!is_subclass_of("Box", "Box")) {
    echo "string:default-false\n";
}
if (!is_subclass_of("BOX", "box", true)) {
    echo "string:allowed-exact-false\n";
}
if (!is_subclass_of("Missing", "Box", true)) {
    echo "missing-source:false\n";
}
if (!is_subclass_of($box, "Missing")) {
    echo "missing-target:false\n";
}

$call = "is_subclass_of";
if (!$call($box, "BOX")) {
    echo "dynamic:false\n";
}
