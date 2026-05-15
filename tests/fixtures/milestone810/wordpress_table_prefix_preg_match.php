<?php
$prefix = 'wp_';
echo preg_match('|[^a-z0-9_]|i', $prefix) ? 'bad' : 'ok';
echo '|';
echo preg_match('|[^a-z0-9_]|i', 'wp-Bad', $matches);
echo '|';
echo $matches[0];
