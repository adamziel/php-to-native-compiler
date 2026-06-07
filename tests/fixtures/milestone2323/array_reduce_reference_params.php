<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function reduce_first_ref(&$carry, $value) {
    return $carry + $value;
}

function reduce_second_ref($carry, &$value) {
    return $carry + $value;
}

$closure = function(&$carry, $value) {
    return $carry + $value;
};

class Reducer {
    public static function stat(&$carry, $value) {
        return $carry + $value;
    }

    public function inst($carry, &$value) {
        return $carry + $value;
    }
}

echo array_reduce([1, 2], "reduce_first_ref", 0), "\n";
echo array_reduce([1, 2], "reduce_second_ref", 0), "\n";
echo array_reduce([1, 2], $closure, 0), "\n";
echo array_reduce([1, 2], ["Reducer", "stat"], 0), "\n";
echo array_reduce([1, 2], [new Reducer(), "inst"], 0), "\n";

foreach ($messages as $index => $message) {
    if ($index > 0) {
        echo "\n";
    }
    echo $message;
}
