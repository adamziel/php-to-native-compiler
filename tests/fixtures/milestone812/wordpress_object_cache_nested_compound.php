<?php
class WP_Object_Cache {
    public $cache;
}

$cache = new WP_Object_Cache();
$cache->cache = array(
    'default' => array(
        'hits' => 2,
    ),
);
$group = 'default';
$key = 'hits';
$offset = 3;
$cache->cache[$group][$key] += $offset;
echo $cache->cache['default']['hits'];
