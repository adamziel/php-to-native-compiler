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

    public function get_row($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_fetch_object($result);
    }

    public function get_prepared_row($query, $name) {
        $result = mysqli_execute_query($this->dbh, $query, array($name));
        return mysqli_fetch_object($result);
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
add_option_row('theme_mods', 'theme-db', 'on');

$direct = $wpdb->get_row("SELECT * FROM wp_options WHERE option_name = 'siteurl'");
echo 'direct=' . $direct->option_id . ':' . $direct->option_name . ':' . $direct->option_value . ':' . $direct->autoload;

echo '|';
$prepared = $wpdb->get_prepared_row(
    'SELECT * FROM `' . $wpdb->options . '` WHERE `option_name` = ?',
    'theme_mods'
);
echo 'prepared=' . $prepared->option_id . ':' . $prepared->option_name . ':' . $prepared->option_value . ':' . $prepared->autoload;

echo '|';
$missing = $wpdb->get_prepared_row('SELECT * FROM wp_options WHERE option_name = ?', 'missing');
echo 'missing=' . ($missing === false ? 'false' : 'row');
