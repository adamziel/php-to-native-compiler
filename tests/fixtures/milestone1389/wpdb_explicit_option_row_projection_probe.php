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

    public function get_results($query, $name) {
        $result = mysqli_execute_query($this->dbh, $query, array($name));
        return mysqli_fetch_all($result, MYSQLI_ASSOC);
    }

    public function get_col($query, $name, $column) {
        $result = mysqli_execute_query($this->dbh, $query, array($name));
        $values = array();
        $value = mysqli_fetch_column($result, $column);
        while ($value !== false) {
            $values[] = $value;
            $value = mysqli_fetch_column($result, $column);
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

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
add_option_row('theme_mods', 'theme-db', 'on');

$direct = $wpdb->get_row("SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = 'siteurl'");
echo 'row=' . $direct->option_id . ':' . $direct->option_name . ':' . $direct->option_value . ':' . $direct->autoload;

echo '|';
$results = $wpdb->get_results(
    'SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `' . $wpdb->options . '` WHERE `option_name` = ?',
    'theme_mods'
);
echo 'results=' . $results[0]['option_id'] . ':' . $results[0]['option_name'] . ':' . $results[0]['autoload'];

echo '|';
$col = $wpdb->get_col(
    'SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ?',
    'theme_mods',
    2
);
echo 'col=' . $col[0];

echo '|';
$missing = $wpdb->get_results(
    'SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ?',
    'missing'
);
echo 'missing=' . (empty($missing) ? 'empty' : 'rows');
