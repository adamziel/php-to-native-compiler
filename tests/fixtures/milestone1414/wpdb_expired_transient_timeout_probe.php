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

    public function get_col($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $names = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $names[] = $row['option_name'];
        }
        return $names;
    }

    public function get_col_stmt($query, $pattern, $threshold) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern, $threshold));
        $result = mysqli_stmt_get_result($stmt);
        $names = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $names[] = $row['option_name'];
        }
        return $names;
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function names_from_result($result) {
    $names = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $names[] = $row['option_name'];
    }
    return implode(',', $names);
}

$wpdb = new wpdb('wp_');
add_option_row('_transient_timeout_update_plugins', '100', 'no');
add_option_row('_transient_timeout_update_themes', '250', 'no');
add_option_row('_transient_timeout_update_core', '900', 'no');
add_option_row('_transient_update_plugins', 'payload', 'no');
add_option_row('_site_transient_timeout_update_core', '150', 'no');
add_option_row('siteurl', 'https://example.test', 'yes');

$direct = mysqli_query(
    $wpdb->dbh,
    "SELECT option_name FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300 ORDER BY option_name"
);
echo 'direct=' . names_from_result($direct);
echo '|';

$prepared = $wpdb->get_col(
    'SELECT `option_name` FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ? AND `option_value` < ? ORDER BY `option_name` ASC',
    array('\_site_transient\_timeout\_%', '300')
);
echo 'prepared=' . implode(',', $prepared);
echo '|';

$stmt = $wpdb->get_col_stmt(
    'SELECT option_name FROM wp_options WHERE option_name LIKE ? AND option_value < ? ORDER BY option_name',
    '_transient_timeout_%',
    101
);
echo 'stmt=' . implode(',', $stmt);
