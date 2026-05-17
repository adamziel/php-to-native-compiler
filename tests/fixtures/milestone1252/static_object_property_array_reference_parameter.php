<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
    }
}

$cache = new WP_Object_Cache();
$seen = 'start';
Cache_Marker::tag($seen, 'var');
$cache->cache['options']['alloptions'] = 'cold';
Cache_Marker::tag($cache->cache['options']['alloptions'], 'static');
echo $seen, '|', $cache->cache['options']['alloptions'];
