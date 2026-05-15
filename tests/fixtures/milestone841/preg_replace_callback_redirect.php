<?php
function _wp_sanitize_utf8_in_redirect($matches) {
    return urlencode($matches[0]);
}

$regex = '/
(
    (?: [\xC2-\xDF][\x80-\xBF]        # double-byte sequences
    |   \xE0[\xA0-\xBF][\x80-\xBF]
    |   [\xE1-\xEC][\x80-\xBF]{2}
    |   \xED[\x80-\x9F][\x80-\xBF]
    |   [\xEE-\xEF][\x80-\xBF]{2}
    |   \xF0[\x90-\xBF][\x80-\xBF]{2}
    |   [\xF1-\xF3][\x80-\xBF]{3}
    |   \xF4[\x80-\x8F][\x80-\xBF]{2}
){1,40}                              # ...one or more times
)/x';

echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/wp-admin/install.php');
echo '|';
echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/påth/é.php');
