<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function execute($query, $params) {
        return mysqli_execute_query($this->dbh, $query, $params);
    }

    public function get_var($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $row = mysqli_fetch_assoc($result);
        if ($row) {
            return $row['option_value'];
        }
        return false;
    }
}

function set_transient_probe($transient, $value, $timeout) {
    global $wpdb;
    $query =
        'INSERT INTO `' . $wpdb->options . '` (`option_name`, `option_value`, `autoload`) ' .
        'VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE ' .
        '`option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)';

    $value_name = '_transient_' . $transient;
    $timeout_name = '_transient_timeout_' . $transient;

    $value_result = $wpdb->execute($query, array($value_name, $value, 'no'));
    $value_affected = mysqli_affected_rows($wpdb->dbh);

    $timeout_result = $wpdb->execute($query, array($timeout_name, $timeout, 'no'));
    $timeout_affected = mysqli_affected_rows($wpdb->dbh);

    if ($value_result && $timeout_result) {
        return 'ok:' . $value_affected . ':' . $timeout_affected;
    }
    return 'failed';
}

$wpdb = new wpdb('wp_');

echo 'first=' . set_transient_probe('update_plugins', 'plugin-v1', '111');
echo '|';
echo 'second=' . set_transient_probe('update_plugins', 'plugin-v2', '222');
echo '|';
echo 'value=' . $wpdb->get_var(
    'SELECT `option_value` FROM `' . $wpdb->options . '` WHERE `option_name` = ? LIMIT 1',
    array('_transient_update_plugins')
);
echo '|';
echo 'timeout=' . $wpdb->get_var(
    'SELECT option_value FROM wp_options WHERE option_name = ? LIMIT 1',
    array('_transient_timeout_update_plugins')
);
echo '|';

$rows = $wpdb->execute(
    'SELECT option_name, option_value FROM wp_options WHERE option_name IN (?, ?)',
    array('_transient_update_plugins', '_transient_timeout_update_plugins')
);
$parts = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $parts[] = $row['option_name'] . '=' . $row['option_value'];
}
echo 'rows=' . implode(',', $parts);
