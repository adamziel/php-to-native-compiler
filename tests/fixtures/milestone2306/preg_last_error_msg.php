<?php
echo preg_last_error_msg(), "\n";
preg_match('/a/', 'a', $m, 0, 99);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
ini_set('pcre.backtrack_limit', '1');
preg_match('/(?:\D+|<\d+>)*[!?]/', 'foobar foobar foobar');
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
$text = json_decode('"\u2019"');
preg_match('/\b/u', $text, $m, 0, 1);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
$text = "VA\xff";
$text .= "LID";
preg_match('/\b/u', $text, $m, 0, 0);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
preg_match('/a/', 'a');
echo preg_last_error(), "|", preg_last_error_msg();
