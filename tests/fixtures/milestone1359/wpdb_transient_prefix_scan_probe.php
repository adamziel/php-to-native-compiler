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

    public function get_col($query) {
        $result = mysqli_query($this->dbh, $query);
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_value'];
        }
        return $values;
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function set_transient($name, $value, $timeout) {
    add_option_row('_transient_' . $name, $value, 'no');
    add_option_row('_transient_timeout_' . $name, $timeout, 'no');
}

function scan_transient_rows() {
    global $wpdb;
    $rows = $wpdb->get_results(
        "SELECT option_name, option_value FROM " . $wpdb->options .
        " WHERE option_name LIKE '_transient_%'"
    );
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_name'] . '=' . $row['option_value'];
    }
    return implode(',', $parts);
}

function scan_timeout_values() {
    global $wpdb;
    $values = $wpdb->get_col(
        "SELECT option_value FROM `" . $wpdb->options .
        "` WHERE `option_name` LIKE '\\_transient_timeout\\_%'"
    );
    return implode(',', $values);
}

function scan_transient_star_rows() {
    global $wpdb;
    $rows = $wpdb->get_results(
        "SELECT * FROM `" . $wpdb->options .
        "` WHERE `option_name` LIKE '\\_transient\\_%'"
    );
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_id'] . ':' . $row['option_name'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
set_transient('update_plugins', 'plugin-payload', '12345');
set_transient('update_themes', 'theme-payload', '67890');

echo 'rows=' . scan_transient_rows();
echo '|';
echo 'timeouts=' . scan_timeout_values();
echo '|';
echo 'star=' . scan_transient_star_rows();
