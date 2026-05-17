<?php
$path = __DIR__ . "/offset_length_payload.inc";
echo file_get_contents($path, false, null, 3, 5);
echo "|";
echo file_get_contents($path, false, null, -5, 4);
echo "|";
set_include_path(__DIR__);
echo file_get_contents("offset_length_payload.inc", true, null, 11, 4);
