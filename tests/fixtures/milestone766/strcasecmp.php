<?php
echo strcasecmp("UTF-8", "utf-8") === 0 ? "same" : "diff";
echo "\n";
echo strcasecmp("abc", "ABD") < 0 ? "lt" : "not";
echo "\n";
echo strcasecmp("beta", "ALPHA") > 0 ? "gt" : "not";
echo "\n";
echo strcasecmp(123, "123") === 0 ? "coerced" : "no";
