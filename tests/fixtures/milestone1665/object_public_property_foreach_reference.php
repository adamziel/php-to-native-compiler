<?php
class Milestone1665Box {
    public $first = "one";
    public $second = "two";
}

$box = new Milestone1665Box();
foreach ($box as $key => &$value) {
    $value = $value . ":" . $key;
}
echo $box->first, "|", $box->second, "|", $value, "|", $key, "\n";
$box->second = "direct";
echo $value, "|";
$value = "tail";
echo $box->second, "|", $value, "\n";
unset($value);

$std = new stdClass();
$alpha = "alpha";
$beta = "beta";
$gamma = "gamma";
$std->{$alpha} = "a";
$std->{$beta} = "b";
foreach ($std as $key => &$value) {
    $value = $key . "=" . $value;
    if ($key === "alpha") {
        $std->{$gamma} = "g";
    }
}
echo $std->alpha, "|", $std->beta, "|", $std->gamma, "|", $value, "|", $key, "\n";
$std->{$gamma} = "direct-g";
echo $value, "|";
$value = "tail-g";
echo $std->gamma, "|", $value;
