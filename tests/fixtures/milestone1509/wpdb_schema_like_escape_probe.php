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
$wpdb->query("CREATE TABLE wp_like_escape_probe (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, payload varchar(64) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10)), KEY literal_percent (payload(20))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$wild = $wpdb->get_results("SHOW INDEX FROM wp_like_escape_probe WHERE Key_name LIKE 'meta_%'");
echo 'wild=', count($wild), ':';
foreach ($wild as $index) {
    echo $index['Key_name'], ';';
}

$escaped = $wpdb->get_results("SHOW INDEX FROM wp_like_escape_probe WHERE `Key_name` LIKE 'meta!_%' ESCAPE '!'");
echo '|escaped=', count($escaped), ':';
foreach ($escaped as $index) {
    echo $index['Key_name'], ':', $index['Column_name'], ';';
}

$literal = $wpdb->get_results("SHOW KEYS FROM wp_like_escape_probe WHERE Key_name LIKE 'literal!_%' ESCAPE '!'");
echo '|literal=', count($literal), ':', $literal[0]['Key_name'], ':', $literal[0]['Sub_part'];
