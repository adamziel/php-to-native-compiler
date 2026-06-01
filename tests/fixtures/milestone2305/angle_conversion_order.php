<?php
var_dump(deg2rad(23));
var_dump(deg2rad(1000));
var_dump(rad2deg(4294967295));
ob_start();
var_dump(rad2deg(-2147483649));
echo rtrim(ob_get_clean(), "\n");
