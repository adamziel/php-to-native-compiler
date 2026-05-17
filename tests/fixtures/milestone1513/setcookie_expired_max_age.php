<?php
setcookie("wordpress_delete", "gone", 1, "/");
setrawcookie("wordpress_raw_delete", "raw gone", 1700000000, "/wp-admin", "Example.TEST", true, true);
$headers = headers_list();
echo count($headers);
echo "\n";
echo $headers[0];
echo "\n";
echo $headers[1];
