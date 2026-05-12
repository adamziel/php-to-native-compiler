<?php
const APP_NAME = "compiler", APP_VERSION = 2, APP_SCALE = 1 + 2 * 3;
CONST APP_FLAGS = ["env" => "dev", "nested" => ["x" => 1]], APP_EMPTY = [];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "|", defined("APP_EMPTY"), "\n";
$copy = APP_FLAGS;
$copy["env"] = "prod";
echo $copy["env"], "|", APP_FLAGS["env"], "|", APP_FLAGS["nested"]["x"], "\n";
function read_grouped_const() {
    return APP_NAME . ":" . APP_VERSION . ":" . APP_FLAGS["nested"]["x"];
}
echo read_grouped_const(), "\n";
