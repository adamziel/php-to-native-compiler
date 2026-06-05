<?php
$call = "wordwrap";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo $call("The quick brown fox", 10, "/");
echo "|";
echo wordwrap("abcdefghijk", 4, "/", true);
echo "|";
echo bin2hex(wordwrap("ab\r\ncd ef", 4, "/"));
