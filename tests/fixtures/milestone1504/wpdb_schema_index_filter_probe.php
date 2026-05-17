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
$wpdb->query("CREATE TABLE wp_index_filter_probe (ID bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, post_content longtext NOT NULL, PRIMARY KEY  (ID), UNIQUE KEY option_name (option_name), KEY meta_lookup (meta_key(191), meta_value(10)), FULLTEXT KEY content_search (post_content)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$exact = $wpdb->get_results("SHOW INDEXES FROM `wp_index_filter_probe` WHERE Key_name = 'meta_lookup'");
echo 'exact=', count($exact), ':';
foreach ($exact as $index) {
    echo $index['Key_name'], ':', $index['Seq_in_index'], ':', $index['Column_name'], ':', $index['Sub_part'], ';';
}

$like = $wpdb->get_results("SHOW INDEX FROM wp_index_filter_probe WHERE `Key_name` LIKE 'content_%'");
echo '|like=', count($like), ':';
foreach ($like as $index) {
    echo $index['Key_name'], ':', $index['Column_name'], ':', $index['Index_type'], ';';
}

$primary = $wpdb->get_results("SHOW KEYS FROM wp_index_filter_probe WHERE `Key_name` = 'PRIMARY'");
echo '|primary=', count($primary), ':', $primary[0]['Key_name'], ':', $primary[0]['Non_unique'];

$missing = $wpdb->get_results("SHOW INDEX FROM wp_index_filter_probe WHERE Key_name LIKE 'missing_%'");
echo '|missing=', count($missing);
