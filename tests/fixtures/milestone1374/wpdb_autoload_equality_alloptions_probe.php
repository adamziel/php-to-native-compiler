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

    public function get_results_prepared($query, $autoload) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $autoload);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_results_execute_query($query, $autoload) {
        $result = mysqli_execute_query($this->dbh, $query, array($autoload));
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

function wp_load_alloptions_legacy() {
    global $wpdb;
    return $wpdb->get_results(
        "SELECT option_name, option_value FROM " . $wpdb->options . " WHERE autoload = 'yes'"
    );
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
add_option_row('blogname', 'Example Blog', 'yes');
add_option_row('home', 'https://home.test', 'no');
add_option_row('theme_mods', 'theme-db', 'auto-on');

echo 'legacy=' . row_parts(wp_load_alloptions_legacy());
echo '|';
echo 'prepared=' . row_parts($wpdb->get_results_prepared(
    'SELECT `option_name`, `option_value` FROM `' . $wpdb->options . '` WHERE `autoload` = ?',
    'auto-on'
));
echo '|';
echo 'execute=' . row_parts($wpdb->get_results_execute_query(
    'SELECT option_name, option_value FROM ' . $wpdb->options . ' WHERE autoload = ?',
    'no'
));
