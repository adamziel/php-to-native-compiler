<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function find_ref(&$value, &$key) {
    echo "fn:$value:$key\n";
    $value = 99;
    $key = "changed";
    return true;
}

class RefChecker {
    public static function staticFind(&$value, &$key) {
        echo "static-string:$value:$key\n";
        $value = 66;
        $key = "static-string";
        return true;
    }

    public static function staticAny(&$value, &$key) {
        echo "static:$value:$key\n";
        $value = 77;
        $key = "static";
        return true;
    }

    public function instanceAll(&$value, &$key) {
        echo "object:$value:$key\n";
        $value = 55;
        $key = "object";
        return true;
    }
}

$items = ["a" => 1, "b" => 2];
$closure = function(&$value, &$key) {
    echo "closure:$value:$key\n";
    $value = 88;
    $key = "closure";
    return true;
};

var_dump(array_find($items, "find_ref"));
var_dump(array_find_key($items, $closure));
var_dump(array_find($items, "RefChecker::staticFind"));
var_dump(array_any($items, ["RefChecker", "staticAny"]));
$checker = new RefChecker();
var_dump(array_all(["z" => 3], [$checker, "instanceAll"]));
var_dump(array_any([[2, 1]], "sort"));
echo implode(",", array_keys($items)), "|", implode(",", $items), "\n";

foreach ($messages as $index => $message) {
    if ($index > 0) {
        echo "\n";
    }
    echo $message;
}
