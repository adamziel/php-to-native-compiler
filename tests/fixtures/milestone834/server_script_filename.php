<?php
echo isset($_SERVER['SCRIPT_FILENAME']) ? 'script-set' : 'script-missing';
echo '|';
echo $_SERVER['SCRIPT_FILENAME'];
