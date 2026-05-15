<?php
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/wp-admin/install.php');
echo '|';
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/path/<bad>"quote"');
echo '|';
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/p%C3%A5th/%C3%A9.php');
