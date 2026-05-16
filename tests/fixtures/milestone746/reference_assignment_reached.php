<?php
function bind_alias($value) {
    echo "before\n";
    $alias =& $value;
    $alias = 2;
    echo $value;
}

bind_alias(1);
