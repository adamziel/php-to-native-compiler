<?php
for ($i = 0; $i < 4; $i = $i + 1) {
    switch ($i) {
        case 1:
            echo "skip:";
            continue 2;
        default:
            echo $i, ":";
    }
    echo "after:";
}
