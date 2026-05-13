<?php
var_dump(6 & 3);
var_dump(6 | 3);
var_dump(6 ^ 3);
var_dump((1 + 2) & 6);
var_dump(1 == 1 & 0);
var_dump(1 | 2 && false);

var_dump("ab" & "AB");
var_dump("A@" | " !");
var_dump("az" ^ "  ");
var_dump("ABC" & "xy");
var_dump("A" | "CD");
var_dump("az" ^ " ");

var_dump("6" & 3);
var_dump(8 | "2");
var_dump("7" ^ true);
var_dump(null | 2);
var_dump(false | 2);
var_dump(true & 3);

$value = 0;
$result = ($value = 4) & 6;
var_dump($result);
echo $value;
