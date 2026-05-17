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
$wpdb->query("CREATE TABLE wp_status_literal_in_alpha (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$wpdb->query("CREATE TABLE wp_status_literal_in_beta (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("CREATE TABLE wp_status_literal_in_other (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin");

$in = $wpdb->get_results("SHOW TABLE STATUS WHERE Name IN ('wp_status_literal_in_beta', 'wp_status_literal_in_alpha')");
echo 'in=', count($in), ':';
foreach ($in as $row) {
    echo $row['Name'], ':', $row['Collation'], ';';
}

$ticked = $wpdb->get_results("SHOW TABLE STATUS WHERE `Name` IN ('wp_status_literal_in_missing', 'wp_status_literal_in_beta')");
echo '|ticked=', count($ticked), ':', $ticked[0]['Name'];
