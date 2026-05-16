<?php
echo php_sapi_name();
echo "|";
echo PHP_SAPI;
echo "|";
$call = "php_sapi_name";
echo function_exists($call) ? "exists" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call();
