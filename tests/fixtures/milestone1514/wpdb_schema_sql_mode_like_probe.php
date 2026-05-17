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
$wpdb->query("CREATE TABLE wp_sql_mode_like_probe (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$default = $wpdb->get_results("SHOW INDEX FROM wp_sql_mode_like_probe WHERE Key_name LIKE 'meta\\_%'");
echo 'default=', count($default), ':';
foreach ($default as $index) {
    echo $index['Key_name'], ';';
}

$wpdb->query("SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = $wpdb->get_results("SHOW INDEX FROM wp_sql_mode_like_probe WHERE Key_name LIKE 'meta\\_%'");
echo '|mode=', count($mode);

$explicit = $wpdb->get_results("SHOW INDEX FROM wp_sql_mode_like_probe WHERE `Key_name` LIKE 'meta!_%' ESCAPE '!'");
echo '|explicit=', count($explicit), ':';
foreach ($explicit as $index) {
    echo $index['Key_name'], ':', $index['Column_name'], ';';
}
