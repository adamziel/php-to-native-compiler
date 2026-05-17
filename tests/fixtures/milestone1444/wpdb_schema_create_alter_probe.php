<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function get_results($query) {
        return mysqli_query($this->dbh, $query);
    }
}

function collect_columns($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Field'] . ':' . $row['Type'] . ':' . $row['Key'] . ':' . $row['Default'];
    }
    return implode(',', $parts);
}

function collect_indexes($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Key_name'] . ':' . $row['Column_name'] . ':' . $row['Non_unique'] . ':' . $row['Visible'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

echo $wpdb->query("CREATE TABLE wp_phptest (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', payload longtext NOT NULL, PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci") ? 'created' : 'failed';
echo ':' . mysqli_affected_rows($wpdb->dbh);

$wpdb->query("ALTER TABLE wp_phptest ADD COLUMN checksum varchar(64) NOT NULL default '', ADD UNIQUE KEY checksum (checksum)");
echo ':' . mysqli_affected_rows($wpdb->dbh);
echo '|';

$tables = $wpdb->get_results("SHOW TABLES LIKE 'wp_phptest'");
$table = mysqli_fetch_row($tables);
echo 'table=' . mysqli_num_rows($tables) . ':' . $table[0];
echo '|';

$describe = $wpdb->get_results('DESCRIBE `wp_phptest`');
echo 'columns=' . mysqli_num_fields($describe) . ':' . collect_columns($describe);
echo '|';

$indexes = $wpdb->get_results('SHOW INDEX FROM wp_phptest');
echo 'indexes=' . mysqli_num_rows($indexes) . ':' . collect_indexes($indexes);
