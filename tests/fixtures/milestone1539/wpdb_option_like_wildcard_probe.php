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

    public function get_col($query) {
        $result = mysqli_query($this->dbh, $query);
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'];
        }
        return $values;
    }
}

function option_names($names) {
    return implode(',', $names);
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'wildcard=' . option_names($wpdb->get_col("SELECT option_name FROM wp_options WHERE option_name LIKE '_transient_%'"));
echo '|escaped=' . option_names($wpdb->get_col("SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_%'"));
echo '|custom=' . option_names($wpdb->get_col("SELECT option_name FROM wp_options WHERE option_name LIKE '!_site!_transient!_%' ESCAPE '!'"));
