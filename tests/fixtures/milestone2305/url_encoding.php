<?php
echo urlencode("a b~+"), "\n";
echo rawurlencode("a b~+"), "\n";
echo urldecode("a+b%2B%7E"), "\n";
echo rawurldecode("a+b%2B%7E"), "\n";
echo bin2hex(rawurldecode("%FF%20"));
