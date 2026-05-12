<?php
const APP_NAME = "compiler";
CONST APP_VERSION = 2;
const APP_SCALE = 1 + 2 * 3;
const APP_ITEMS = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "\n";
echo constant("APP_NAME"), "|", defined("APP_ITEMS"), "|", defined("MISSING_CONST"), "\n";
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo count($copy), "|", $copy["name"], "|", APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "\n";
function read_declared_const() {
    return APP_NAME . ":" . APP_VERSION;
}
echo read_declared_const(), "\n";
$name = "APP_NAME";
echo constant($name), "\n";
