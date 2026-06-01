<?php
$hebrew_text = "The hebrev function converts logical Hebrew text to visual text.\nThe function tries to avoid breaking words.\n";
var_dump(hebrev($hebrew_text));
var_dump(hebrev($hebrew_text, 15));
$heb = chr(224) . chr(225) . "(x)" . chr(226);
echo bin2hex(hebrev($heb)), "\n";
echo str_replace("\n", "<n>", hebrev("abc def\n", 5));
