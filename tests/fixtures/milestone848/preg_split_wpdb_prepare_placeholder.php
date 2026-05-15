<?php
$allowed_format = '(?:[1-9][0-9]*[$])?[-+0-9]*(?: |0|\'.)?[-+0-9]*(?:\.[0-9]+)?';
$pattern = "/(^|[^%]|(?:%%)+)(%(?:$allowed_format)?[sdfFi])/";
$split = preg_split(
    $pattern,
    'INSERT INTO wp_posts VALUES (%1$s, %05d, %.2f, %i)',
    -1,
    PREG_SPLIT_DELIM_CAPTURE
);
echo count($split), "\n";
echo $split[0], "\n";
echo $split[1], "\n";
echo $split[2], "\n";
echo $split[3], "\n";
echo $split[4], "\n";
echo $split[5], "\n";
echo $split[6], "\n";
echo $split[7], "\n";
echo $split[8], "\n";
echo $split[9], "\n";
echo $split[10], "\n";
echo $split[11], "\n";
echo $split[12];
