<?php
echo isset($_SERVER['HTTP_HOST']) ? 'host-set' : 'host-missing';
echo '|';
echo $_SERVER['HTTP_HOST'];
