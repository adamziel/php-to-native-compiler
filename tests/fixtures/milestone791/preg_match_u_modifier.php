<?php
$utf8_pcre = true;
if (preg_match('//u', '') !== 1) {
    $utf8_pcre = false;
}
echo $utf8_pcre ? 'utf8' : 'fallback';
