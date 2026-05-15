<?php
function determine_charset() {
    $charset = 'utf8mb4';
    $collate = 'utf8mb4_unicode_ci';
    return compact('charset', 'collate');
}

$result = determine_charset();
echo $result['charset'];
echo '|';
echo $result['collate'];
