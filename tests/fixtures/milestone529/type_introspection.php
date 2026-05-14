<?php
class Box {}

$box = new Box();
$values = [null, false, 7, 3.5, "x", ["nested"], $box];
foreach ($values as $value) {
    echo gettype($value), "\n";
}

echo is_null(null) ? "1" : "0", is_null(0) ? "1" : "0", "\n";
echo is_bool(false) ? "1" : "0", is_bool(0) ? "1" : "0", "\n";
echo is_int(7) ? "1" : "0", is_integer(7) ? "1" : "0", is_long(7) ? "1" : "0", is_int("7") ? "1" : "0", "\n";
echo is_float(3.5) ? "1" : "0", is_double(3.5) ? "1" : "0", is_float(3) ? "1" : "0", "\n";
echo is_string("x") ? "1" : "0", is_string(1) ? "1" : "0", "\n";
echo is_array(["x"]) ? "1" : "0", is_array($box) ? "1" : "0", "\n";
echo is_scalar(false) ? "1" : "0", is_scalar(1) ? "1" : "0", is_scalar(1.5) ? "1" : "0", is_scalar("x") ? "1" : "0", is_scalar(null) ? "1" : "0", is_scalar(["x"]) ? "1" : "0", is_scalar($box) ? "1" : "0", "\n";

$call = "gettype";
echo $call(true), "\n";
$predicate = "is_array";
echo $predicate(["dynamic"]) ? "1" : "0";
