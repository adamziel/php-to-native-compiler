<?php
function show_call($label, $callable, $value) {
    try {
        $callable($value);
    } catch (TypeError $e) {
        echo $label, ": ", $e->getMessage(), "\n";
    }
}

show_call("dynamic-count", "array_count_values", 42);
show_call("dynamic-sum", "array_sum", "items");
show_call("dynamic-product", "array_product", true);

try {
    array_count_values(null);
} catch (TypeError $e) {
    echo "direct-count: ", $e->getMessage(), "\n";
}

try {
    array_sum(false);
} catch (TypeError $e) {
    echo "direct-sum: ", $e->getMessage(), "\n";
}

try {
    array_product(new stdClass());
} catch (TypeError $e) {
    echo "direct-product: ", $e->getMessage();
}
