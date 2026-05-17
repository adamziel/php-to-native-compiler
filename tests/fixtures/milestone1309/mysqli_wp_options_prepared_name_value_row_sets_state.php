<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function get_results_prepared($query) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb('wp_');
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$all = $wpdb->get_results_prepared('SELECT option_name, option_value FROM wp_options');
$parts = array();
foreach ($all as $row) {
    $parts[] = $row['option_name'] . '=' . $row['option_value'];
}
echo 'all=' . implode(',', $parts);
echo '|';
$autoload = $wpdb->get_results_prepared("SELECT `option_name`, `option_value` FROM `wp_options` WHERE `autoload` IN ( 'yes', 'on', 'auto-on', 'auto' )");
$parts = array();
foreach ($autoload as $row) {
    $parts[] = $row['option_name'] . '=' . $row['option_value'];
}
echo 'autoload=' . implode(',', $parts);
echo '|';
$named = mysqli_execute_query($wpdb->dbh, "SELECT `option_name`, `option_value` FROM `wp_options` WHERE `option_name` IN ('theme_mods','missing','home')");
$first = mysqli_fetch_assoc($named);
$second = mysqli_fetch_assoc($named);
echo 'named=' . mysqli_num_rows($named) . ':' . $first['option_name'] . '=' . $first['option_value'] . ',' . $second['option_name'] . '=' . $second['option_value'];
