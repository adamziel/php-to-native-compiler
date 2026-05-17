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
        $result = mysqli_query($this->dbh, $query);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NULL, metadata varchar(20) NULL, PRIMARY KEY  (meta_id), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("CREATE TABLE wpXprobeXmeta (id bigint(20) unsigned NOT NULL, PRIMARY KEY  (id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");

$tables = mysqli_query($wpdb->dbh, "SHOW TABLES LIKE 'wp_probe_meta'");
echo 'underscore=';
while ($table = mysqli_fetch_row($tables)) {
    echo $table[0], ';';
}

echo '|escaped=';
$escaped = mysqli_query($wpdb->dbh, "SHOW TABLES LIKE 'wp\\_probe\\_meta'");
while ($table = mysqli_fetch_row($escaped)) {
    echo $table[0], ';';
}

echo '|status=';
$status = $wpdb->get_results("SHOW TABLE STATUS LIKE 'wp\\_probe\\_met_'");
foreach ($status as $row) {
    echo $row['Name'], ':', $row['Collation'], ';';
}

echo '|columns=';
$columns = $wpdb->get_results("SHOW FULL COLUMNS FROM `wp_probe_meta` LIKE 'meta\\_%'");
foreach ($columns as $column) {
    echo $column['Field'], ';';
}

echo '|where=';
$where = $wpdb->get_results("SHOW COLUMNS FROM wp_probe_meta WHERE Field LIKE 'meta__d'");
foreach ($where as $column) {
    echo $column['Field'], ';';
}
