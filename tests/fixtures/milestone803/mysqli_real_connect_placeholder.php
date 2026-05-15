<?php
$handle = mysqli_init();
$ok = mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
echo $ok ? 'connected' : 'failed';
echo '|';
echo get_class($handle);
echo '|';
echo $handle->connect_errno;
echo '|';
echo $handle->connect_error === null ? 'null' : 'set';
