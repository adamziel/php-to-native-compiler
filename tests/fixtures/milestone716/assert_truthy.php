<?php
echo assert(true) ? "1" : "0";
echo assert(1, "bootstrap") ? "1" : "0";
$call = "assert";
echo $call(class_exists("Exception"), null) ? "1" : "0";
