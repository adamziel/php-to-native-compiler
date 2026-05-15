<?php
function wp_cache_get($key, $group = '', $force = false, &$found = null) {
    echo isset($found) ? 'found-set' : 'found-null';
    return false;
}

$notoptions = wp_cache_get('notoptions', 'options');
echo '|';
echo $notoptions === false ? 'miss' : 'hit';
