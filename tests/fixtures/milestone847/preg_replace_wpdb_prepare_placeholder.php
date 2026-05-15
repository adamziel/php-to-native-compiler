<?php
$allowed_format = '(?:[1-9][0-9]*[$])?[-+0-9]*(?: |0|\'.)?[-+0-9]*(?:\.[0-9]+)?';
echo preg_replace(
    "/%(?:%|$|(?!($allowed_format)?[sdfFi]))/",
    '%%\\1',
    'SELECT %s, %05d, 100%, %q, %%s, %1$s'
);
echo "\n";
echo preg_replace(
    "/%(?:%|$|(?!($allowed_format)?[sdfFi]))/",
    '%%\\1',
    'LIKE %foo% AND rate %1$q'
);
