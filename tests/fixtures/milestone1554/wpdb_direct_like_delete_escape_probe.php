<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function affected_rows() {
        return mysqli_affected_rows($this->dbh);
    }

    public function option_parts() {
        $result = mysqli_query($this->dbh, 'SELECT option_name, option_value FROM wp_options');
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'] . '=' . $row['option_value'];
        }
        return implode(',', $values);
    }
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'site-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'wild=';
echo $wpdb->query("DELETE FROM wp_options WHERE option_name LIKE '_transient_%'") ? $wpdb->affected_rows() : 'failed';

$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");

echo '|escape=';
echo $wpdb->query("DELETE FROM `wp_options` WHERE `option_name` LIKE '!_transient!_%' ESCAPE '!'") ? $wpdb->affected_rows() : 'failed';
echo '|left=' . $wpdb->option_parts();
