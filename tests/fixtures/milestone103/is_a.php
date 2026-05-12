<?php
class Box {}
class Crate {}

$box = new box();
if (is_a($box, "Box")) {
    echo "object:box\n";
}
if (is_a($box, "box")) {
    echo "object:case-insensitive\n";
}
if (!is_a($box, "Crate")) {
    echo "object:not-crate\n";
}
if (!is_a("Box", "Box")) {
    echo "string:default-false\n";
}
if (is_a("BOX", "box", true)) {
    echo "string:allowed\n";
}
if (!is_a("Missing", "Box", true)) {
    echo "missing-source:false\n";
}
if (!is_a($box, "Missing")) {
    echo "missing-target:false\n";
}

$call = "is_a";
if ($call($box, "BOX")) {
    echo "dynamic:object\n";
}
