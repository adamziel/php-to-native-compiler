<?php
$missing = file_get_contents(__DIR__ . "/missing-local-read.txt");
echo $missing === false ? "missing=false" : "missing=value";
echo "|";
$bad_offset = file_get_contents("php://input", false, null, -1);
echo $bad_offset === false ? "offset=false" : "offset=value";
