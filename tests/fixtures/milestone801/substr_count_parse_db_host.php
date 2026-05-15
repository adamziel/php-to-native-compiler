<?php
$host = '2001:db8::1';
echo substr_count($host, ':');
echo '|';
echo substr_count('db.example:3306', ':') > 1 ? 'ipv6' : 'ipv4';
echo '|';
echo substr_count('aaaa', 'aa');
