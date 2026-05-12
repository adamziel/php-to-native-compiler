<?php
function counter() {
    static $count = 0;
    $count = $count + 1;
    return $count;
}
echo counter();
