<?php
function sort_recursive(&$items) {
    foreach ($items as &$item) {
        if (is_array($item)) {
            sort_recursive($item);
        }
    }
}

if (false) {
    $items = [1];
    foreach ($items as &$item) {
        echo $item;
    }
}

echo "registered";
