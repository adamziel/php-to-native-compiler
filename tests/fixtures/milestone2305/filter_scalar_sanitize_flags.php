<?php
var_dump(FILTER_FLAG_STRIP_BACKTICK);
var_dump(filter_var("", FILTER_DEFAULT, array("flags" => FILTER_FLAG_EMPTY_STRING_NULL)));
var_dump(filter_var("``a`b`c``", FILTER_UNSAFE_RAW, FILTER_FLAG_STRIP_BACKTICK));
var_dump(filter_var("\x7f", FILTER_UNSAFE_RAW, FILTER_FLAG_STRIP_HIGH));
var_dump(filter_var("\x7f", FILTER_SANITIZE_ENCODED, FILTER_FLAG_STRIP_HIGH));
var_dump(filter_var("\x7f", FILTER_SANITIZE_SPECIAL_CHARS, FILTER_FLAG_STRIP_HIGH));
var_dump(filter_var("bad", FILTER_VALIDATE_INT, array("options" => array("default" => 321))));
