<?php
$value = 'Ada';
$items = array(&$value, 'name' => &$value);
echo $items[0] . '|' . $items['name'];
