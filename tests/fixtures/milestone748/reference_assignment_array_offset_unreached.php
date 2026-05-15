<?php
function descend(&$items) {
    $cursor =& $items[0];
    return $cursor;
}

if (false) {
    $items = [1];
    $alias =& $items[0];
    echo $alias;
}

echo "registered";
