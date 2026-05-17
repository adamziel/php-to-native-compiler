<?php
function capture_warning($errno, $errstr, $errfile, $errline) {
    echo "handled:" . $errno;
    echo ":" . (str_contains($errstr, "missing-handler-read.txt") ? "path" : "missing");
    echo ":" . basename($errfile);
    echo ":" . $errline;
    return true;
}

set_error_handler("capture_warning", E_WARNING);
$first = file_get_contents("tests/fixtures/missing-handler-read.txt");
echo $first === false ? "|false" : "|value";

class WarningSink {
    public function handle($errno, $errstr, $errfile, $errline) {
        echo "|array:" . $errno;
        echo ":" . (str_contains($errstr, "missing-array-handler-read.txt") ? "path" : "missing");
        echo ":" . $errline;
        return true;
    }
}

$sink = new WarningSink();
set_error_handler(array($sink, "handle"), E_WARNING);
$arrayHandled = file_get_contents("tests/fixtures/missing-array-handler-read.txt");
echo $arrayHandled === false ? "|false" : "|value";

function passthrough_warning($errno, $errstr) {
    echo "|passthrough:" . $errno;
    echo ":" . (str_contains($errstr, "missing-passthrough-read.txt") ? "path" : "missing");
    return false;
}

error_reporting(0);
set_error_handler("passthrough_warning", E_WARNING);
$second = file_get_contents("tests/fixtures/missing-passthrough-read.txt");
echo $second === false ? "|false" : "|value";

restore_error_handler();
$third = file_get_contents("tests/fixtures/missing-quiet-read.txt");
echo $third === false ? "|quiet-false" : "|quiet-value";
