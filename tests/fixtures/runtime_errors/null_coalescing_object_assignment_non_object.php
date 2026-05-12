<?php
function fallback() {
    echo "non-object-called\n";
    return "fallback";
}
$number = 42;
$number->value ??= fallback();
