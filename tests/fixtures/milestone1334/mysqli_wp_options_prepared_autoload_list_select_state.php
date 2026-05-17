<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function prepare_two($query, $one, $two) {
        return array($query, array($one, $two));
    }

    public function get_results_prepared($prepared) {
        $stmt = mysqli_prepare($this->dbh, $prepared[0]);
        $one = $prepared[1][0];
        $two = $prepared[1][1];
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

$wpdb = new wpdb('wp_');
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");

$prepared = $wpdb->prepare_two(
    'SELECT option_name, option_value, autoload FROM ' . $wpdb->options . ' WHERE autoload IN (?, ?)',
    'yes',
    'on'
);
$rows = $wpdb->get_results_prepared($prepared);
$parts = array();
foreach ($rows as $row) {
    $parts[] = $row['option_name'] . ':' . $row['autoload'];
}
echo 'prepared=' . implode(',', $parts);

echo '|';
$direct = mysqli_execute_query(
    $wpdb->dbh,
    'SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `autoload` IN (?, ?)',
    array('on', 'yes')
);
$direct_parts = array();
while ($row = mysqli_fetch_assoc($direct)) {
    $direct_parts[] = $row['option_id'] . ':' . $row['option_name'] . ':' . $row['autoload'];
}
echo 'direct=' . implode(',', $direct_parts);
