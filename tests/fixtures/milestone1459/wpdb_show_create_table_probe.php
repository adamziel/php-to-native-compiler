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
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_links (link_id bigint(20) unsigned NOT NULL auto_increment, link_url varchar(255) NOT NULL default '', link_name varchar(255) NOT NULL default '', link_visible varchar(20) NOT NULL default 'Y', PRIMARY KEY  (link_id), KEY link_visible (link_visible), KEY link_name (link_name(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_probe_links ADD COLUMN link_updated datetime NOT NULL default '0000-00-00 00:00:00', DROP KEY link_visible, ADD KEY visible_name (link_visible, link_name(191))");

$row = $wpdb->get_row('SHOW CREATE TABLE `wp_probe_links`');

echo $row['Table'];
echo "\n";
echo $row['Create Table'];
