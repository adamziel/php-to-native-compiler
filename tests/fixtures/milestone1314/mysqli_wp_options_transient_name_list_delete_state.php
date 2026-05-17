<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function get_results($query) {
        $result = mysqli_query($this->dbh, $query);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

function wp_cache_init() {
    $GLOBALS['wp_object_cache'] = array();
}

function wp_cache_set($key, $value, $group = '') {
    global $wp_object_cache;
    $wp_object_cache[$group][$key] = $value;
    return true;
}

function wp_cache_get($key, $group = '', $force = false, &$found = null) {
    global $wp_object_cache;
    if (isset($wp_object_cache[$group][$key])) {
        $found = true;
        return $wp_object_cache[$group][$key];
    }
    $found = false;
    return false;
}

function wp_cache_delete($key, $group = '') {
    global $wp_object_cache;
    unset($wp_object_cache[$group][$key]);
    return true;
}

function delete_transient_probe($transient) {
    global $wpdb;
    $value_name = '_transient_' . $transient;
    $timeout_name = '_transient_timeout_' . $transient;
    $query = "DELETE FROM " . $wpdb->options . " WHERE option_name IN ('" . $value_name . "','" . $timeout_name . "')";
    $result = $wpdb->query($query);
    wp_cache_delete($value_name, 'transient');
    return $result ? true : false;
}

$wpdb = new wpdb('wp_');
wp_cache_init();
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_feed_mod', 'cached-feed', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '123456', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
wp_cache_set('_transient_feed_mod', 'cached-feed', 'transient');
$found = null;
echo wp_cache_get('_transient_feed_mod', 'transient', false, $found);
echo ':';
echo $found ? 'cache' : 'miss';
echo '|';
echo delete_transient_probe('feed_mod') ? 'deleted' : 'failed';
echo ':';
echo mysqli_affected_rows($wpdb->dbh);
echo '|';
$found = null;
$cached = wp_cache_get('_transient_feed_mod', 'transient', false, $found);
echo $found ? $cached : 'cache-miss';
echo '|';
$rows = $wpdb->get_results("SELECT option_name, option_value FROM wp_options WHERE option_name IN ('_transient_feed_mod','_transient_timeout_feed_mod','siteurl')");
$parts = array();
foreach ($rows as $row) {
    $parts[] = $row['option_name'] . '=' . $row['option_value'];
}
echo 'rows=' . implode(',', $parts);
