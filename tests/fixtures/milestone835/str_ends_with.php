<?php
$_SERVER['SCRIPT_FILENAME'] = '/index.php';
echo str_ends_with($_SERVER['SCRIPT_FILENAME'], 'php.cgi') ? 'cgi' : 'not-cgi';
echo '|';
echo str_ends_with('index.php', '.php') ? 'suffix' : 'missing';
echo '|';
echo str_ends_with('index.php', '') ? 'empty' : 'missing';
