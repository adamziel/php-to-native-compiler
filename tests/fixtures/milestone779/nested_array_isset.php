<?php
$ini_all = false;
$setting = 'memory_limit';
echo isset($ini_all[$setting]['access']) ? 'set' : 'unset';

$ini_all = [];
$ini_all[$setting] = [];
$ini_all[$setting]['access'] = 7;
echo '|';
echo isset($ini_all[$setting]['access']) ? 'set' : 'unset';

$ini_all[$setting]['access'] = null;
echo '|';
echo isset($ini_all[$setting]['access']) ? 'set' : 'unset';
