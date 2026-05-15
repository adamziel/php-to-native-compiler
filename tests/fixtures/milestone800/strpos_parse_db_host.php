<?php
$host = 'db.example:3306';
$colon = strpos($host, ':');
echo $colon === false ? 'none' : $colon;
echo '|';
echo strpos($host, ':', $colon + 1) === false ? 'no-more' : 'more';
echo '|';
echo strpos($host, '') === 0 ? 'empty' : 'bad';
