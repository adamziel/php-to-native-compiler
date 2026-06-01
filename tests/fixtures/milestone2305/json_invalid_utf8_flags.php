<?php
$bad = "a\xb0b";
echo json_encode($bad, JSON_INVALID_UTF8_IGNORE), "|", json_last_error(), ":", json_last_error_msg(), "\n";
echo json_encode($bad, JSON_INVALID_UTF8_SUBSTITUTE), "|", json_last_error(), ":", json_last_error_msg(), "\n";

$json = "\"a\xb0b\"";
var_dump(json_decode($json, true, 512, JSON_INVALID_UTF8_IGNORE));
var_dump(json_decode($json, true, 512, JSON_INVALID_UTF8_SUBSTITUTE));

$outside = "[\xb0]";
var_dump(json_decode($outside, true, 512, JSON_INVALID_UTF8_IGNORE));
echo json_last_error(), ":", json_last_error_msg();
