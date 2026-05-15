<?php
echo preg_match('/^Microsoft-IIS\//', 'Microsoft-IIS/10.0');
echo '|';
echo preg_match('/^Microsoft-IIS\//', 'phpc');
echo '|';
echo preg_match('/php$/', 'index.php');
echo '|';
echo preg_match('/dex/', 'index.php');
