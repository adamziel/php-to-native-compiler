<?php
try {
    echo "try";
} catch (Throwable|Exception $e) {
    echo "catch";
} finally {
    echo "finally";
}
