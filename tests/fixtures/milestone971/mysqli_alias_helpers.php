<?php
$handle = mysqli_init();
echo mysqli_set_opt($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true) ? "set-opt" : "failed";
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_escape_string($handle, "quote'\"\\");
