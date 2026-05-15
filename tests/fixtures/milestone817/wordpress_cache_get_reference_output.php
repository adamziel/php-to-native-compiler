<?php
class WP_Object_Cache {
    public function get($key, $group = 'default', $force = false, &$found = null) {
        $found = false;
        return false;
    }
}

function wp_cache_get($key, $group = '', $force = false, &$found = null) {
    global $wp_object_cache;

    $value = $wp_object_cache->get($key, $group, $force, $found);
    echo isset($found) ? ($found ? 'hit' : 'miss') : 'unset';
    return $value;
}

$wp_object_cache = new WP_Object_Cache();
$notoptions = wp_cache_get('notoptions', 'options');
echo '|';
echo $notoptions === false ? 'false' : 'value';
