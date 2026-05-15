<?php
$allowed = [];
$allowed['group'][] = 'first';
$allowed['group'][] = 'second';
$pos = array_search('first', $allowed['group'], true);

unset($allowed['group'][$pos]);
unset($allowed['group']['missing']);
unset($allowed['missing']['child']);

if (array_key_exists(0, $allowed['group'])) {
    echo "first:set\n";
} else {
    echo "first:unset\n";
}

echo "second:", $allowed['group'][1];
