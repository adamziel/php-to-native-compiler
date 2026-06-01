<?php
echo "*** Testing hebrev() : basic functionality ***\n";
$hebrew_text = "The hebrev function converts logical Hebrew text to visual text.\nThe function tries to avoid breaking words.\n";
var_dump(hebrev($hebrew_text));
var_dump(hebrev($hebrew_text, 15));
echo rtrim(hebrev(".abc\n.def\n", -1), "\n");
