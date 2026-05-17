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

    public function get_col_prepared($query, $pattern) {
        $result = mysqli_execute_query($this->dbh, $query, array($pattern));
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'];
        }
        return $values;
    }

    public function get_name_values_prepared($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $pattern);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'] . '=' . $row['option_value'];
        }
        return $values;
    }
}

function option_parts($parts) {
    return implode(',', $parts);
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'wildcard=' . option_parts($wpdb->get_col_prepared(
    'SELECT option_name FROM wp_options WHERE option_name LIKE ?',
    '_transient_%'
));
echo '|escaped=' . option_parts($wpdb->get_name_values_prepared(
    'SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ?',
    "\\_transient\\_%"
));
