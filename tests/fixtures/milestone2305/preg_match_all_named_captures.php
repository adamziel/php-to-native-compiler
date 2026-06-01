<?php
preg_match('/(?P<word>[a-z]+)-(?P<num>\d+)/', 'ab-12', $one, PREG_OFFSET_CAPTURE);
echo implode(",", array_keys($one)), "|", $one['word'][0], ":", $one['word'][1], "|", $one['num'][0], ":", $one['num'][1], "\n";
$count = preg_match_all('/(?P<word>[a-z]+)-(?P<num>\d+)/', 'ab-12 cd-34', $matches, PREG_PATTERN_ORDER | PREG_OFFSET_CAPTURE);
echo $count, "|", implode(",", array_keys($matches)), "\n";
echo $matches['word'][1][0], ":", $matches['word'][1][1], "|";
echo $matches[1][0][0], ":", $matches[1][0][1], "|";
echo $matches['num'][0][0], ":", $matches['num'][0][1], "\n";
preg_match_all('/(?P<a>a)(?P<b>b)?/', 'a ab', $set, PREG_SET_ORDER);
echo implode(",", array_keys($set[0])), "|", implode(",", array_keys($set[1])), "\n";
echo array_key_exists('b', $set[0]) ? "b" : "no-b";
echo "|", array_key_exists('a', $set[1]) ? "a" : "no-a";
echo "|", $set[1]['b'];
