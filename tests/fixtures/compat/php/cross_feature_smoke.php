<?php
const PREFIX = 'wp';
define('ANSWER', 42);

function keep_even($value) {
    return $value % 2 == 0;
}

function label($items, $suffix = 'ok') {
    return PREFIX . ':' . $suffix . ':' . array_sum($items);
}

class Box {
    public $value;
}

$values = [1, 2, 3];
$values[] = 4;
$even = array_filter($values, 'keep_even');

echo label($even), "\n";

$box = new Box();
$box->value = array_sum($even);

if (isset($box->value) && $box->value === 6) {
    echo get_class($box), "\n";
}

foreach ($even as $key => $value) {
    echo $key, '=', $value, "\n";
}

echo defined('ANSWER') ? constant('ANSWER') : 'missing';
