<?php
try {
    throw new Exception();
} catch (Exception $e) {
    echo "catch";
} finally {
    echo "finally";
}
echo "after";
