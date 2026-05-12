<?php
class Box {
    public $name;
    protected $secret;
    private static $cache;
}

$box = new box();
if (property_exists($box, "name")) {
    echo "object:name\n";
}
if (property_exists($box, "secret")) {
    echo "object:secret\n";
}
if (property_exists($box, "cache")) {
    echo "object:static\n";
}
if (property_exists("BOX", "cache")) {
    echo "class:static\n";
}
if (!property_exists("Box", "missing")) {
    echo "class:missing\n";
}
if (!property_exists("Missing", "name")) {
    echo "missing-class:false\n";
}

$call = "property_exists";
if ($call($box, "name")) {
    echo "dynamic:exists\n";
}
