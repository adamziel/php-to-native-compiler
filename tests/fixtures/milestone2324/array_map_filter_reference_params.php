<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function map_ref(&$value) {
    $value = "mapped-" . $value;
    return $value;
}

function filter_ref(&$value, &$key) {
    $value = "local-" . $value;
    $key = "local-" . $key;
    return true;
}

$closure = function(&$value) {
    return "closure-" . $value;
};

class CallbackBox {
    public static function stat(&$value) {
        return true;
    }

    public function inst(&$value) {
        return "inst-" . $value;
    }
}

$items = ["x" => "one", "y" => "two"];
$mapped = array_map("map_ref", $items);
echo $mapped["x"], "|", $mapped["y"], "|", $items["x"], "\n";

$filtered = array_filter(["k" => "keep"], "filter_ref", ARRAY_FILTER_USE_BOTH);
echo count($filtered), "|", $filtered["k"], "\n";

$closure_result = array_map($closure, ["c" => "three"]);
echo $closure_result["c"], "\n";

$static = array_filter(["s" => "four"], ["CallbackBox", "stat"]);
$instance = array_map([new CallbackBox(), "inst"], ["i" => "five"]);
echo count($static), "|", $instance["i"], "\n";

foreach ($messages as $index => $message) {
    if ($index > 0) {
        echo "\n";
    }
    echo $message;
}
