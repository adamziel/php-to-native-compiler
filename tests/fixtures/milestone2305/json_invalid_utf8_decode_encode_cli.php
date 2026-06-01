<?php
$bad = '"a' . chr(0xb0) . 'b"';
var_dump(json_decode($bad, true, 512, JSON_INVALID_UTF8_IGNORE));
var_dump(bin2hex(json_decode($bad, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)));

$raw = "a" . chr(0xf0) . chr(0x80) . chr(0x80) . "A";
var_dump(json_encode($raw, JSON_INVALID_UTF8_IGNORE));
var_dump(json_encode($raw, JSON_INVALID_UTF8_SUBSTITUTE));

try {
    json_decode('"abc"', true, -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
