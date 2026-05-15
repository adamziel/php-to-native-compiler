<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$result = mysqli_query($handle, 'SELECT @@SESSION.sql_mode');
echo $result === false ? 'false' : 'result';
echo '|';
echo empty($result) ? 'empty' : 'set';
