<?php
function rhs_value() {
    echo "rhs\n";
    return "value";
}

$value = 1;
echo ($value->name = rhs_value());
