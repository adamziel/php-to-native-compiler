<?php
$one = "\x61\xb0\x62";
var_dump(json_encode($one));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_encode($one, JSON_INVALID_UTF8_IGNORE));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_encode($one, JSON_INVALID_UTF8_SUBSTITUTE));
var_dump(json_last_error(), json_last_error_msg());
echo bin2hex(json_encode($one, JSON_UNESCAPED_UNICODE | JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$overlong = "\x61\xf0\x80\x80\x41";
var_dump(json_encode($overlong, JSON_INVALID_UTF8_IGNORE));
var_dump(json_encode($overlong, JSON_INVALID_UTF8_SUBSTITUTE));
echo bin2hex(json_encode($overlong, JSON_UNESCAPED_UNICODE | JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$array = array($one, "ok");
echo json_encode($array, JSON_INVALID_UTF8_IGNORE), "\n";
echo json_encode($array, JSON_INVALID_UTF8_SUBSTITUTE), "\n";
echo json_encode($one, JSON_INVALID_UTF8_IGNORE | JSON_INVALID_UTF8_SUBSTITUTE);
