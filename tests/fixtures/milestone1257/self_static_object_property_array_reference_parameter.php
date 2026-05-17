<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
    }

    public static function mark_self($cache) {
        self::tag($cache->cache['options']['alloptions'], 'self');
    }

    public static function mark_static($cache) {
        static::tag($cache->cache['options']['alloptions'], 'static');
    }
}

class Child_Cache_Marker extends Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ':child-' . $suffix;
    }
}

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
Cache_Marker::mark_self($cache);
Child_Cache_Marker::mark_static($cache);
echo $cache->cache['options']['alloptions'];
