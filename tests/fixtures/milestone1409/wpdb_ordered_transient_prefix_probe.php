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

    public function get_results($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_results_stmt($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function row_parts($rows) {
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_name'] . '=' . $row['option_value'];
    }
    return implode(',', $parts);
}

function star_parts($rows) {
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_id'] . ':' . $row['option_name'] . ':' . $row['autoload'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
add_option_row('_transient_update_themes', 'theme-payload', 'no');
add_option_row('_transient_timeout_update_themes', '555', 'no');
add_option_row('_site_transient_update_core', 'core-payload', 'no');
add_option_row('_site_transient_timeout_update_core', '777', 'no');

$direct = mysqli_query(
    $wpdb->dbh,
    "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%' ORDER BY option_name"
);
echo 'direct=' . row_parts(array(mysqli_fetch_assoc($direct), mysqli_fetch_assoc($direct)));
echo '|';

$prepared = $wpdb->get_results(
    'SELECT `option_name`, `option_value`, `autoload` FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ? ORDER BY `option_name` ASC',
    array('\_site_transient\_%')
);
echo 'prepared=' . row_parts($prepared);
echo '|';

$stmt = $wpdb->get_results_stmt(
    'SELECT * FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ? ORDER BY `option_name`',
    '_transient_timeout_%'
);
echo 'stmt=' . star_parts($stmt);
