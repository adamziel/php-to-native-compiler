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

    public function get_results($query) {
        $result = mysqli_query($this->dbh, $query);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_results_prepared($query, $one, $two) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 'ss', $one, $two);
        mysqli_stmt_execute($stmt);
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

function set_transient($transient, $value, $timeout) {
    add_option_row('_transient_' . $transient, $value, 'no');
    add_option_row('_transient_timeout_' . $transient, $timeout, 'no');
}

function get_transient_rows($transient) {
    global $wpdb;
    return $wpdb->get_results_prepared(
        'SELECT * FROM `' . $wpdb->options . '` WHERE `option_name` IN (?, ?)',
        '_transient_' . $transient,
        '_transient_timeout_' . $transient
    );
}

function prime_autoload_options() {
    global $wpdb;
    $rows = mysqli_execute_query(
        $wpdb->dbh,
        'SELECT * FROM `wp_options` WHERE `autoload` IN (?, ?)',
        array('yes', 'on')
    );
    $loaded = array();
    while ($row = mysqli_fetch_assoc($rows)) {
        $loaded[$row['option_name']] = $row['option_id'] . ':' . $row['option_value'];
    }
    return $loaded;
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
add_option_row('theme_mods', 'theme-db', 'on');
set_transient('update_plugins', 'plugin-payload', '12345');

$site = $wpdb->get_row("SELECT * FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
echo 'single=' . $site->option_id . ':' . $site->option_name . ':' . $site->option_value . ':' . $site->autoload;

echo '|';
$transient_rows = get_transient_rows('update_plugins');
$parts = array();
foreach ($transient_rows as $row) {
    $parts[] = $row['option_id'] . ':' . $row['option_name'] . ':' . $row['option_value'];
}
echo 'transient=' . implode(',', $parts);

echo '|';
$all = $wpdb->get_results('SELECT * FROM wp_options');
$names = array();
foreach ($all as $row) {
    $names[] = $row['option_id'] . ':' . $row['option_name'];
}
echo 'all=' . implode(',', $names);

echo '|';
$autoload = prime_autoload_options();
echo 'autoload=' . $autoload['siteurl'] . ',' . $autoload['theme_mods'];
