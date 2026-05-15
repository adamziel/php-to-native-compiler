<?php
$existing = 'yes';
global $existing, $wp_filter;
echo $existing;
echo '|';
echo isset($wp_filter) ? 'set' : 'unset';
echo '|';
echo $wp_filter === null ? 'null' : 'not-null';
echo '|';
echo $wp_filter ? 'truthy' : 'falsey';
