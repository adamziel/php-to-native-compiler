<?php

ob_start();

$from = SplFixedArray::fromArray(array(1 => "one", 3 => false));
var_dump($from);

$resized = new SplFixedArray(3);
$resized[0] = "slot";
$resized->setSize(2);
var_dump($resized);

echo rtrim(ob_get_clean(), "\n");
