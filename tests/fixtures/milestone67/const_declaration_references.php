<?php
define("RUNTIME_BASE", 3);
const FROM_DEFINE = RUNTIME_BASE + 1;
const BASE = "compiler";
const VERSION = 2, DOUBLE_VERSION = VERSION * 2, LABEL = BASE . ":" . DOUBLE_VERSION;
const FILTER_MODE = ARRAY_FILTER_USE_BOTH;
const ITEMS = [BASE => LABEL, "mode" => FILTER_MODE, "key-mode" => ARRAY_FILTER_USE_KEY, "from-define" => FROM_DEFINE];
echo LABEL, "|", FILTER_MODE, "|", ITEMS["compiler"], "|", ITEMS["mode"], "|", ITEMS["key-mode"], "|", ITEMS["from-define"], "\n";
function read_referenced_const() {
    return LABEL . ":" . ARRAY_FILTER_USE_KEY;
}
echo read_referenced_const(), "\n";
$name = "DOUBLE_VERSION";
echo constant($name), "|", FROM_DEFINE, "\n";
