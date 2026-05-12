<?php
define("RUNTIME_FACTOR", 3);
const BASE = "compiler";

function describe($label = BASE . ":" . ARRAY_FILTER_USE_KEY, $factor = RUNTIME_FACTOR + 1, $items = [BASE => ARRAY_FILTER_USE_BOTH]) {
    echo $label, "|", $factor, "|", $items["compiler"], "\n";
}

describe();
describe("manual", 9, ["compiler" => "override"]);

function late_default($value = LATE_DEFAULT) {
    return $value;
}

const LATE_DEFAULT = "late";
echo late_default(), "\n";
