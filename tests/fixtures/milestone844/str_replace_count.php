<?php
function deep_replace($search, $subject) {
    $count = 1;
    while ($count) {
        $subject = str_replace($search, '', $subject, $count);
    }
    return $subject;
}

echo 'deep=', deep_replace('%0D', '%0%0%0DDD'), "\n";
$count = 99;
echo str_replace('na', '', 'banana', $count), '|', $count, "\n";
$count = 99;
echo str_replace('z', 'x', 'banana', $count), '|', $count, "\n";
$count = 99;
echo str_replace('', 'x', 'banana', $count), '|', $count;
