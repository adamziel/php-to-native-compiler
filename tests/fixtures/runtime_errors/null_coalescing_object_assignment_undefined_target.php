<?php
function fallback() {
    echo "undefined-target-called\n";
    return "fallback";
}
$missing_box->value ??= fallback();
