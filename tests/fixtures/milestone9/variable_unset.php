<?php
$value = "Ada";
$nullable = null;

unset($value);
unset($missing);
unset($nullable);

if (isset($value)) {
    echo "value:set\n";
} else {
    echo "value:unset\n";
}
if (empty($missing)) {
    echo "missing:empty\n";
} else {
    echo "missing:not-empty\n";
}
if (isset($nullable)) {
    echo "nullable:set\n";
} else {
    echo "nullable:unset\n";
}

function demo($name) {
    unset($name);
    if (isset($name)) {
        echo "local:set\n";
    } else {
        echo "local:unset\n";
    }
}

$name = "global";
demo("local");
echo "global=", $name, "\n";

$value = "Bea";
echo "reassigned=", $value, "\n";
