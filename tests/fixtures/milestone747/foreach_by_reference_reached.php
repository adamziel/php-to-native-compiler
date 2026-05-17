<?php
function &items() {
    static $items = [1];
    return $items;
}

foreach (items() as &$item) {
    echo $item;
}
