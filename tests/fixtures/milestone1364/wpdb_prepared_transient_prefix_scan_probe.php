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

    public function get_results_prepared($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_col_prepared($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $pattern);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
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

function set_site_transient($name, $value, $timeout) {
    add_option_row('_site_transient_' . $name, $value, 'no');
    add_option_row('_site_transient_timeout_' . $name, $timeout, 'no');
}

function scan_prepared_transient_rows() {
    global $wpdb;
    $rows = $wpdb->get_results_prepared(
        'SELECT option_name, option_value FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ?',
        '\_transient\_%'
    );
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_name'] . '=' . $row['option_value'];
    }
    return implode(',', $parts);
}

function scan_prepared_timeout_values() {
    global $wpdb;
    $values = $wpdb->get_col_prepared(
        'SELECT option_value FROM ' . $wpdb->options . ' WHERE option_name LIKE ?',
        '_transient_timeout_%'
    );
    return implode(',', $values);
}

function scan_prepared_site_star_rows() {
    global $wpdb;
    $rows = $wpdb->get_results_prepared(
        'SELECT * FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ?',
        '\_site_transient\_%'
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
set_site_transient('update_core', 'core-payload', '11111');

echo 'rows=' . scan_prepared_transient_rows();
echo '|';
echo 'timeouts=' . scan_prepared_timeout_values();
echo '|';
echo 'site-star=' . scan_prepared_site_star_rows();
