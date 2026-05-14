<?php
$value = 2;
if ($value == 1):
    echo "one";
elseif ($value == 2):
    echo "two";
elseif ($missing):
    echo "missing";
else:
    echo "else";
endif;
echo "\n";

$flag = false;
if ($flag):
    echo "flag";
else:
    echo "fallback";
endif;
echo "\n";

function probe($value) {
    if ($value):
        return "yes";
    else:
        return "no";
    endif;
}

echo probe(true), "\n";
echo probe(false);
