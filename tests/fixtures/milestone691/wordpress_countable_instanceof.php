<?php
function _wp_countable_probe($value) {
    if (is_array($value) || $value instanceof Countable) {
        return "countable";
    }

    return "plain";
}

echo _wp_countable_probe([1, 2, 3]), "\n";
echo _wp_countable_probe("Ada");
