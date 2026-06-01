<?php
$encoded = urlencode("A1_-.~ +/%");
echo $encoded, "\n";
echo urldecode($encoded), "\n";
foreach (["%41+%2B%2f%25", "bad%2g%", "%00%FF%7E+"] as $value) {
    echo bin2hex(urldecode($value)), "\n";
}
$decode = "urldecode";
echo $decode("name=WordPress+Core%2BCLI");
