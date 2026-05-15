<?php
function bind_alias($value) {
    echo "before\n";
    $alias =& $value;
    echo "after";
}

bind_alias(1);
