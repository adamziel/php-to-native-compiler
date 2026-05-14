<?php
echo extension_loaded("mbstring") ? "1" : "0";
echo extension_loaded("MBSTRING") ? "1" : "0";
echo extension_loaded("json") ? "1" : "0";
echo "\n";

$call = "extension_loaded";
echo $call("simplexml") ? "1" : "0";
