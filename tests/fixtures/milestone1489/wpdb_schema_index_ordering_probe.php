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
$wpdb->query("CREATE TABLE wp_order_probe (lookup_id bigint(20) unsigned NOT NULL auto_increment, object_id bigint(20) unsigned NOT NULL default 0, meta_key varchar(191) NOT NULL default '', updated_at datetime NOT NULL default '0000-00-00 00:00:00', PRIMARY KEY  (lookup_id), KEY object_recent (object_id ASC, updated_at DESC), KEY meta_recent (meta_key(100) DESC)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_order_probe ADD KEY object_meta_recent (object_id, meta_key(100) ASC, updated_at DESC)");

$indexes = $wpdb->get_results("SHOW INDEX FROM wp_order_probe");
echo count($indexes), ':';
foreach ($indexes as $index) {
    echo $index['Key_name'], ':', $index['Seq_in_index'], ':', $index['Column_name'], ':', $index['Sub_part'], ':', $index['Collation'], ';';
}

$created = mysqli_query($wpdb->dbh, 'SHOW CREATE TABLE wp_order_probe');
$row = mysqli_fetch_assoc($created);
echo '|';
echo $row['Create Table'];
