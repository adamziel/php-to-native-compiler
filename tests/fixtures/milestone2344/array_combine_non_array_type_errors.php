<?php

foreach ([42, false] as $candidate) {
    try {
        var_dump(array_combine($candidate, []));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}

$call = "array_combine";
foreach ([42, null] as $index => $candidate) {
    try {
        var_dump($call([], $candidate));
    } catch (TypeError $e) {
        echo $e->getMessage();
        if ($index === 0) {
            echo "\n";
        }
    }
}
