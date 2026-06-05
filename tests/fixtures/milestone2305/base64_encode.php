<?php
$binary = "A" . chr(0) . chr(255);
echo base64_encode("hello world"), "\n";
echo base64_encode("f"), "|", base64_encode("fo"), "|", base64_encode("foo"), "\n";
echo bin2hex(base64_decode(base64_encode($binary), true)), "\n";
$call = "base64_encode";
echo function_exists($call) ? "yes" : "no";
echo "|", is_callable($call) ? "callable" : "missing";
echo "|", $call(42042);
