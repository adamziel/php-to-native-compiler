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

    public function get_row($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_fetch_assoc($result);
    }

    public function get_var($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_num_rows($result);
    }
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NULL, PRIMARY KEY  (meta_id), KEY meta_key (meta_key(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");

$status = $wpdb->get_row("SHOW TABLE STATUS LIKE 'wp_probe_meta'");
echo $status['Name'];
echo '|';
echo $status['Engine'];
echo '|';
echo $status['Rows'];
echo '|';
echo $status['Collation'];
echo '|';
echo $status['Create_options'];
echo '|where=';
echo $wpdb->get_var("SHOW TABLE STATUS WHERE Name = 'wp_probe_meta'");
echo '|missing=';
echo $wpdb->get_var("SHOW TABLE STATUS LIKE 'wp_missing_meta'");
