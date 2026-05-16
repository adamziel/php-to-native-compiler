<?php
$items = [[1]];
foreach ($items[0] as &$item) {
    echo $item;
}
