<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function get_row($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_fetch_assoc($result);
    }

    public function get_results($query) {
        return mysqli_query($this->dbh, $query);
    }
}

function collect_schema_fields($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Field'] . ':' . $row['Type'] . ':' . $row['Key'] . ':' . $row['Extra'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

$tables = $wpdb->get_results("SHOW TABLES LIKE 'wp_options'");
$table = mysqli_fetch_row($tables);
echo 'table=' . mysqli_num_rows($tables) . ':' . $table[0];
echo '|';

$describe = $wpdb->get_results('DESCRIBE wp_options;');
echo 'describe=' . mysqli_num_fields($describe) . ':' . collect_schema_fields($describe);
echo '|';

$columns = $wpdb->get_results('SHOW FULL COLUMNS FROM `wp_options`');
while ($column = mysqli_fetch_assoc($columns)) {
    if ($column['Field'] === 'autoload') {
        echo 'autoload=' . $column['Type'] . ':' . $column['Default'] . ':' . $column['Collation'];
    }
}
