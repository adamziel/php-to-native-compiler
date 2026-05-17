<?php
function first_warning($errno, $errstr) {
    echo "first:" . $errno;
    echo ":" . (str_contains($errstr, "missing-first-after-restore.txt") ? "path" : "missing");
    return true;
}

function second_warning($errno, $errstr) {
    echo "second:" . $errno;
    echo ":" . (str_contains($errstr, "missing-second-top.txt") ? "path" : "missing");
    return true;
}

set_error_handler("first_warning", E_WARNING);
$previous = set_error_handler("second_warning", E_WARNING);
echo is_string($previous) ? "prev=" . $previous : "prev=other";

$top = file_get_contents("tests/fixtures/missing-second-top.txt");
echo $top === false ? "|top-false" : "|top-value";

restore_error_handler();
$restored = file_get_contents("tests/fixtures/missing-first-after-restore.txt");
echo $restored === false ? "|restored-false" : "|restored-value";

restore_error_handler();
