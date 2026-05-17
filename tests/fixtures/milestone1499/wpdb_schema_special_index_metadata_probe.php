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
$wpdb->query("CREATE TABLE wp_special_index_probe (ID bigint(20) unsigned NOT NULL auto_increment, post_title text NOT NULL, post_content longtext NOT NULL, geo point NOT NULL, PRIMARY KEY  (ID), FULLTEXT KEY title_content (post_title, post_content), SPATIAL KEY geo_lookup (geo)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_special_index_probe ADD FULLTEXT INDEX content_only (post_content), ADD SPATIAL INDEX geo_recent (geo)");

$indexes = $wpdb->get_results('SHOW INDEX FROM wp_special_index_probe');
echo count($indexes), ':';
foreach ($indexes as $index) {
    echo $index['Key_name'], ':', $index['Seq_in_index'], ':', $index['Column_name'], ':', $index['Non_unique'], ':', $index['Index_type'], ';';
}

$created = mysqli_query($wpdb->dbh, 'SHOW CREATE TABLE wp_special_index_probe');
$created_row = mysqli_fetch_assoc($created);
echo '|';
echo $created_row['Create Table'];
