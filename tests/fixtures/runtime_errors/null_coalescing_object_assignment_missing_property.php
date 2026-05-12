<?php
class Box {
    public $value;
}
function fallback() {
    echo "missing-called\n";
    return "fallback";
}
$box = new Box();
$box->missing ??= fallback();
