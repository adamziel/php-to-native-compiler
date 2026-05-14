<?php
$value = "2";
switch ($value):
    case 1:
        echo "one";
        break;
    case 2;
        echo "two";
    default:
        echo "-default";
    case "tail";
        echo "-tail";
        break;
endswitch;
echo "\n";

$word = "none";
SWITCH ($word):
    DEFAULT;
        echo "fallback";
    CASE "none":
        echo "matched";
        break;
ENDSWITCH;
echo "\n";

$missing = "missing";
switch ($missing):
    default:
        echo "fallback";
    case "after":
        echo "-after";
        break;
endswitch;
