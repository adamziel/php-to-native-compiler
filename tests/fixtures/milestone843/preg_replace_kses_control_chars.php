<?php
echo preg_replace('/[\x00-\x08\x0B\x0C\x0E-\x1F]/', '', 'safe-content');
echo '|';
echo preg_replace('/[\x00-\x08\x0B\x0C\x0E-\x1F]/', '', "keep\t\n\rchars");
