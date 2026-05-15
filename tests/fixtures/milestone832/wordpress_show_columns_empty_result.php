<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$result = mysqli_query($handle, 'SHOW FULL COLUMNS FROM `wp_options`');
$describe = mysqli_query($handle, 'DESCRIBE wp_users;');
echo $result === false ? 'columns-empty' : 'columns-result';
echo '|';
echo $describe === false ? 'describe-empty' : 'describe-result';
echo '|';
echo mysqli_errno($handle);
echo '|';
echo mysqli_error($handle) === '' ? 'clean' : 'dirty';
