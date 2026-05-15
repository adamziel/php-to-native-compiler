<?php
class Bag {
    public $items = array();
}

$bag = new Bag();
$bag->items = array(array('value'), array(0), array(''));
$key = 0;
echo empty($bag->items[0][$key]) ? 'present-empty' : 'present-set';
echo '|';
echo empty($bag->items[1][0]) ? 'zero-empty' : 'zero-set';
echo '|';
echo empty($bag->items[2][0]) ? 'string-empty' : 'string-set';
echo '|';
echo empty($bag->items[3][0]) ? 'missing-empty' : 'missing-set';
