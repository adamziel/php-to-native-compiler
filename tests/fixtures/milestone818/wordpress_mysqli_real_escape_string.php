<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$data = "can't \"stop\" \\ now\n";
echo mysqli_real_escape_string($handle, $data);
