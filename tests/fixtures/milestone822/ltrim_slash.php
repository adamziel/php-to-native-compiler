<?php
$path = '/wp-content/plugins';
$call = 'ltrim';

echo ltrim($path, '/'), "\n";
echo ltrim(" \tabc\n"), "|";
echo ltrim("\r\n\t (SELECT", "\r\n\t ("), "|";
echo $call('//mu-plugins', '/');
