<?php
function trace($name, $value) {
    echo "trace:", $name, "\n";
    return $value;
}

var_dump(true && false);
var_dump(true || false);
var_dump(false || "php");
var_dump("0" || false);
var_dump("php" && [1]);

$left = "start";
false && ($left = "bad-and");
true || ($left = "bad-or");
echo "short:", $left, "\n";
false || ($left = "or-ran");
echo "after-or:", $left, "\n";
true && ($left = "and-ran");
echo "after-and:", $left, "\n";

$symbol_or = false || true;
$word_or = false or true;
$symbol_and = true && false;
$word_and = true and false;
var_dump($symbol_or);
var_dump($word_or);
var_dump($symbol_and);
var_dump($word_and);

var_dump(true || false && false);
var_dump((true || false) && false);
var_dump(false or true and false);

var_dump(false && trace("skipped-and", true));
var_dump(true || trace("skipped-or", false));
var_dump(true && trace("and-rhs", "php"));
var_dump(false || trace("or-rhs", [1]));
