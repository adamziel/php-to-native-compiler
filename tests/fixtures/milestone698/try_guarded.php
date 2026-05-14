<?php
function guarded_try() {
    try {
        echo "try";
    } catch (Throwable|Exception $e) {
        echo "catch";
    } finally {
        echo "finally";
    }
}

echo "after";
