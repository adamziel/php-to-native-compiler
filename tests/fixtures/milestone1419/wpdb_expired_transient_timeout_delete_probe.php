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

    public function delete_expired($query, $params) {
        mysqli_execute_query($this->dbh, $query, $params);
        return mysqli_affected_rows($this->dbh);
    }

    public function delete_expired_stmt($query, $pattern, $threshold) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern, $threshold));
        return mysqli_stmt_affected_rows($stmt) . ':' . mysqli_affected_rows($this->dbh);
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function option_parts($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['option_name'] . '=' . $row['option_value'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb('wp_');
add_option_row('_transient_timeout_update_plugins', '100', 'no');
add_option_row('_transient_timeout_update_themes', '500', 'no');
add_option_row('_transient_update_plugins', 'payload', 'no');
add_option_row('_site_transient_timeout_update_core', '120', 'no');
add_option_row('_site_transient_update_core', 'core-payload', 'no');
add_option_row('siteurl', 'https://example.test', 'yes');

$wpdb->query(
    "DELETE FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300"
);
echo 'direct=' . mysqli_affected_rows($wpdb->dbh);
echo '|';

$prepared = $wpdb->delete_expired(
    'DELETE FROM `wp_options` WHERE `option_name` LIKE ? AND `option_value` < ?',
    array('\_site_transient\_timeout\_%', '300')
);
echo 'prepared=' . $prepared;
echo '|';

$stmt = $wpdb->delete_expired_stmt(
    'DELETE FROM wp_options WHERE option_name LIKE ? AND option_value < ?',
    '_transient_timeout_%',
    600
);
echo 'stmt=' . $stmt;
echo '|';

$rows = mysqli_query($wpdb->dbh, 'SELECT option_name, option_value FROM wp_options');
echo option_parts($rows);
