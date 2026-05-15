<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
echo mysqli_get_server_info($handle);
echo '|';
echo preg_replace('/[^0-9.].*/', '', mysqli_get_server_info($handle));
