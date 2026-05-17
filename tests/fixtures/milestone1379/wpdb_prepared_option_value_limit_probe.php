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

    public function get_var_prepared($query, $name) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $name);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $row = mysqli_fetch_assoc($result);
        if (!$row) {
            return null;
        }
        return $row['option_value'];
    }

    public function get_var_execute_query($query, $name) {
        $result = mysqli_execute_query($this->dbh, $query, array($name));
        $row = mysqli_fetch_assoc($result);
        if (!$row) {
            return null;
        }
        return $row['option_value'];
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function set_transient_probe($transient, $value, $timeout) {
    add_option_row('_transient_' . $transient, $value, 'no');
    add_option_row('_transient_timeout_' . $transient, $timeout, 'no');
}

function get_transient_value_probe($transient) {
    global $wpdb;
    return $wpdb->get_var_prepared(
        'SELECT option_value FROM ' . $wpdb->options . ' WHERE option_name = ? LIMIT 1',
        '_transient_' . $transient
    );
}

function get_transient_timeout_probe($transient) {
    global $wpdb;
    return $wpdb->get_var_execute_query(
        'SELECT `option_value` FROM `' . $wpdb->options . '` WHERE `option_name` = ? LIMIT 1',
        '_transient_timeout_' . $transient
    );
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
set_transient_probe('update_plugins', 'plugin-payload', '12345');

echo 'value=' . get_transient_value_probe('update_plugins');
echo '|';
echo 'timeout=' . get_transient_timeout_probe('update_plugins');
echo '|';
$missing = get_transient_value_probe('missing');
echo 'missing=' . ($missing === null ? 'null' : $missing);
