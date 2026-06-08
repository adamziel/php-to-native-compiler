<?php
$one = "\"a\xb0b\"";
var_dump(json_decode($one));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_decode($one, true, 512, JSON_INVALID_UTF8_IGNORE));
var_dump(json_last_error(), json_last_error_msg());
echo bin2hex(json_decode($one, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)), "\n";
var_dump(json_last_error(), json_last_error_msg());

$overlong = "\"\x61\xf0\x80\x80\x41\"";
echo bin2hex(json_decode($overlong, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$array = json_decode("[\"\xc1\xc1\",\"a\"]", true, 512, JSON_INVALID_UTF8_IGNORE);
var_dump($array);
$substituted = json_decode("[\"\xc1\xc1\",\"a\"]", true, 512, JSON_INVALID_UTF8_SUBSTITUTE);
echo bin2hex($substituted[0]), "|", bin2hex($substituted[1]), "\n";
echo bin2hex(json_decode($one, true, 512, JSON_INVALID_UTF8_IGNORE | JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$outside = "[" . "\xb0" . "]";
var_dump(json_decode($outside, true, 512, JSON_INVALID_UTF8_IGNORE));
echo json_last_error(), "|", json_last_error_msg();
