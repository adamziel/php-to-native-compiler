<?php
$value = 29;
$value %= 5;
var_dump($value);
var_dump($value %= 3);
var_dump($value);

$items = ['count' => 22];
var_dump($items['count'] %= 6);
var_dump($items['count']);

class Box {
    public $value;
}

$box = new Box();
$box->value = 17;
var_dump($box->value %= 5);
var_dump($box->value);

for ($i = 35; $i > 5; $i %= 8) {
    echo $i, ":";
}
