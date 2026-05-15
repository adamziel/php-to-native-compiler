<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
echo mysqli_select_db($handle, 'wordpress') ? 'selected' : 'failed';
echo '|';
echo mysqli_select_db($handle, null) ? 'null-ok' : 'failed';
