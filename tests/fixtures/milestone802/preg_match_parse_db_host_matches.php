<?php
$ipv4 = '#^(?P<host>[^:/]*)(?::(?P<port>[\d]+))?#';
$ipv6 = '#^(?:\[)?(?P<host>[0-9a-fA-F:]+)(?:\]:(?P<port>[\d]+))?#';

$result = preg_match($ipv4, 'db.example:3306', $matches);
echo $result;
echo '|';
echo $matches[0];
echo '|';
echo $matches['host'];
echo '|';
echo $matches['port'];
echo '|';

$result = preg_match($ipv6, '[2001:db8::1]:3306', $matches);
echo $result;
echo '|';
echo $matches[0];
echo '|';
echo $matches['host'];
echo '|';
echo $matches['port'];
echo '|';

$matches = array('old');
$result = preg_match('/missing/', 'db.example:3306', $matches);
echo $result;
echo '|';
echo count($matches);
