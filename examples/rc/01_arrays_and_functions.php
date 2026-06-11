<?php

function score_label($name, $score) {
    if ($score >= 10) {
        return $name . ":hot";
    }
    if ($score >= 5) {
        return $name . ":warm";
    }
    return $name . ":cold";
}

function keep_nonzero($value) {
    return $value;
}

$names = array("compiler", "runtime", "docs");
$scores = array(12, 7, 0);
$scoreboard = array_combine($names, $scores);

echo "scoreboard\n";
foreach ($scoreboard as $name => $score) {
    echo score_label($name, $score), "\n";
}

$kept = array_filter($scoreboard, "keep_nonzero");
print_r(array_chunk($kept, 2, true));
