<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function prepare_three($query, $one, $two, $three) {
        return array($query, array($one, $two, $three));
    }

    public function get_results_prepared($prepared) {
        $stmt = mysqli_prepare($this->dbh, $prepared[0]);
        $one = $prepared[1][0];
        $two = $prepared[1][1];
        $three = $prepared[1][2];
        mysqli_stmt_bind_param($stmt, 'sss', $one, $two, $three);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_object($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb('wp_');
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");

$prepared = $wpdb->prepare_three(
    'SELECT option_name, autoload FROM ' . $wpdb->options . ' WHERE option_name IN (?, ?, ?)',
    'theme_mods',
    'missing',
    'siteurl'
);
$rows = $wpdb->get_results_prepared($prepared);
$parts = array();
foreach ($rows as $row) {
    $parts[] = $row->option_name . ':' . $row->autoload;
}
echo 'names=' . implode(',', $parts);

echo '|';
$direct = mysqli_execute_query(
    $wpdb->dbh,
    'SELECT `option_name`, `autoload` FROM `wp_options` WHERE `autoload` IN (?, ?)',
    array('yes', 'on')
);
$direct_parts = array();
while ($row = mysqli_fetch_assoc($direct)) {
    $direct_parts[] = $row['option_name'] . ':' . $row['autoload'];
}
echo 'autoload=' . implode(',', $direct_parts);
