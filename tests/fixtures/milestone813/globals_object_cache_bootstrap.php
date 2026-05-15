<?php
class WP_Object_Cache {
    public $global_groups = array();
}

function wp_cache_init() {
    $GLOBALS['wp_object_cache'] = new WP_Object_Cache();
}

function wp_cache_add_global_groups($groups) {
    global $wp_object_cache;

    $wp_object_cache->global_groups = array_merge(
        $wp_object_cache->global_groups,
        array_fill_keys((array) $groups, true)
    );
}

wp_cache_init();
wp_cache_add_global_groups(array('users', 'sites'));
echo $GLOBALS['wp_object_cache']->global_groups['users'] ? 'users' : 'missing';
echo '|';
echo $GLOBALS['wp_object_cache']->global_groups['sites'] ? 'sites' : 'missing';
