<?php
class JsonText {
    public function __toString() { return '{"ok":true}'; }
}
class JsonListText {
    public function __toString() { return '[1,2]'; }
}

var_dump(json_decode(new JsonText(), true));
var_dump(json_validate(new JsonText()));

$call = "json_decode";
var_dump($call(new JsonListText(), true));

var_dump(json_decode(null));
var_dump(json_validate(null));

try {
    json_decode([]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    json_validate(new stdClass());
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

$bad = "\"a\xb0b\"";
echo bin2hex(json_decode($bad, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)), "\n";
echo json_validate($bad, 512, JSON_INVALID_UTF8_IGNORE) ? "valid" : "invalid";
