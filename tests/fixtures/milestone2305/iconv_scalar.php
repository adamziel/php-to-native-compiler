<?php
iconv_set_encoding("internal_encoding", "UTF-8");
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=");
$period = base64_decode("44CC");
echo iconv_strlen($japanese), "\n";
echo iconv_strpos($japanese, $period), "\n";
echo iconv_strrpos($japanese, $period), "\n";
echo bin2hex(iconv_substr($japanese, 2, 7, "UTF-8")), "\n";
