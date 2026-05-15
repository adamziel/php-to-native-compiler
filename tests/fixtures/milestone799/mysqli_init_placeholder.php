<?php
$handle = mysqli_init();
echo get_class($handle);
echo '|', $handle->connect_errno;
echo '|', $handle->connect_error === null ? 'null' : 'set';
