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

    public function get_var($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_num_rows($result);
    }
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_options (option_id bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, autoload varchar(20) NOT NULL default 'yes', PRIMARY KEY  (option_id), UNIQUE KEY option_name (option_name)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, option_id bigint(20) unsigned NOT NULL default 0, meta_key varchar(255) NULL, PRIMARY KEY  (meta_id), KEY option_id (option_id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");

$tables = mysqli_query($wpdb->dbh, "SHOW TABLES LIKE 'wp_probe_%'");
echo 'tables=';
while ($table = mysqli_fetch_row($tables)) {
    echo $table[0], ';';
}

echo '|status=';
$status = $wpdb->get_results("SHOW TABLE STATUS LIKE 'wp_probe_%'");
foreach ($status as $row) {
    echo $row['Name'], ':', $row['Collation'], ';';
}

echo '|columns=';
$columns = $wpdb->get_results("SHOW FULL COLUMNS FROM `wp_probe_options` LIKE 'option_%'");
foreach ($columns as $column) {
    echo $column['Field'], ':', $column['Key'], ';';
}

echo '|where=';
$where = $wpdb->get_results("SHOW COLUMNS FROM wp_probe_meta WHERE Field LIKE 'meta_%'");
foreach ($where as $column) {
    echo $column['Field'], ':', $column['Null'], ';';
}

echo '|missing=';
echo $wpdb->get_var("SHOW TABLE STATUS LIKE 'wp_missing_%'");
