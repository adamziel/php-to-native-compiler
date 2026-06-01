<?php
$sep = "a\xE2\x80\xA8b";
$para = "a\xE2\x80\xA9b";
$neighbor = "a\xE2\x80\xA7b";

echo json_encode($sep, JSON_UNESCAPED_UNICODE), "\n";
echo json_encode($sep, JSON_UNESCAPED_LINE_TERMINATORS), "\n";
echo bin2hex(json_encode($sep, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS)), "\n";
echo json_encode($para, JSON_UNESCAPED_UNICODE), "\n";
echo bin2hex(json_encode($para, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS)), "\n";
echo bin2hex(json_encode($neighbor, JSON_UNESCAPED_UNICODE));
