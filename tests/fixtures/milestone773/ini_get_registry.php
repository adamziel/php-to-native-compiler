<?php
$value = ini_get('memory_limit');
echo is_string($value) ? 'string' : 'other';
echo '|';
echo $value === false ? 'false' : 'not-false';
echo '|';
echo ini_get('definitely_missing_phpc_option') === false ? 'missing-false' : 'missing-other';
