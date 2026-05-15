<?php
function deep_replace($search, $subject) {
    $count = 1;
    while ($count) {
        $subject = str_replace($search, '', $subject, $count);
    }
    return $subject;
}

$count = 0;
echo str_replace(array('%0D', '%0A'), '', '%0%0DDD%0A', $count), '|', $count, "\n";
echo 'deep=', deep_replace(array('%0D', '%0A'), '%0%0DDD%0A');
