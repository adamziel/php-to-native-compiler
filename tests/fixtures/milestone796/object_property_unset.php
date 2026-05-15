<?php
class Box
{
    public $name;
    public $other;
}

$box = new Box();
$box->name = 'Ada';
$box->other = 'kept';
unset($box->name);
echo isset($box->name) ? 'set' : 'unset';
$property = 'other';
unset($box->$property);
echo '|', isset($box->other) ? 'set' : 'unset';
