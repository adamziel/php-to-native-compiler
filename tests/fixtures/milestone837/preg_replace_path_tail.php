<?php
echo preg_replace('#/[^/]*$#i', '', '/index.php');
echo '|';
echo preg_replace('#/[^/]*$#i', '', '/wp-admin/admin.php?page=site');
echo '|';
echo preg_replace('#/[^/]*$#i', '', 'index.php');
