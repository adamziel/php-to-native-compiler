<?php
class ParagonIE_Sodium_Compat {}

echo assert(class_exists('ParagonIE_Sodium_Compat'), 'Possible filesystem/autoloader bug?') ? "assert-ok\n" : "assert-fail\n";
echo "after\n";
