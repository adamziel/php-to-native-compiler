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
$wpdb->query("CREATE TABLE wp_dbdelta_probe (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', legacy varchar(20) NOT NULL default 'keep', PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("CREATE TABLE wp_dbdelta_probe (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(200) NOT NULL default '', title text NULL, status varchar(20) NOT NULL default 'publish', PRIMARY KEY  (id), KEY slug (slug(64), title(32)), KEY status (status)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");

$columns = $wpdb->get_results("DESCRIBE wp_dbdelta_probe");
echo 'columns=', count($columns), ':';
foreach ($columns as $column) {
    echo $column['Field'], ':', $column['Type'], ':', $column['Key'], ';';
}

$indexes = $wpdb->get_results("SHOW INDEX FROM wp_dbdelta_probe");
echo '|indexes=', count($indexes), ':';
foreach ($indexes as $index) {
    echo $index['Key_name'], ':', $index['Seq_in_index'], ':', $index['Column_name'], ':', $index['Sub_part'], ';';
}
