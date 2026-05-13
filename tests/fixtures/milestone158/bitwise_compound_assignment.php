<?php
$value = 14;
$value &= 11;
var_dump($value);
$value |= 1;
var_dump($value);
$value ^= 3;
var_dump($value);
$value <<= 2;
var_dump($value);
$value >>= 3;
var_dump($value);

$text = "ab";
$text &= "AB";
var_dump($text);
$text |= " !";
var_dump($text);

$items = ['bits' => 6, 'shift' => 2];
var_dump($items['bits'] &= 3);
var_dump($items['bits']);
var_dump($items['bits'] |= 8);
var_dump($items['bits']);
var_dump($items['shift'] <<= 3);
var_dump($items['shift']);

class Box {
    public $mask;
}

$box = new Box();
$box->mask = 5;
var_dump($box->mask ^= 3);
var_dump($box->mask);
var_dump($box->mask >>= 1);
var_dump($box->mask);

for ($i = 1; $i < 16; $i <<= 1) {
    echo $i;
}
