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

    public function get_results($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_status_in_with_statement($left, $right) {
        $stmt = mysqli_prepare($this->dbh, 'SHOW TABLE STATUS WHERE Name IN (?, ?)');
        mysqli_stmt_execute($stmt, array($left, $right));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb();
$wpdb->query("CREATE TABLE wp_status_in_alpha (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$wpdb->query("CREATE TABLE wp_status_in_beta (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("CREATE TABLE wp_status_in_other (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin");

$in = $wpdb->get_results('SHOW TABLE STATUS WHERE Name IN (?, ?)', array('wp_status_in_beta', 'wp_status_in_alpha'));
echo 'in=', count($in), ':';
foreach ($in as $row) {
    echo $row['Name'], ':', $row['Collation'], ';';
}

$ticked = $wpdb->get_results('SHOW TABLE STATUS WHERE `Name` IN (?, ?)', array('wp_status_in_missing', 'wp_status_in_beta'));
echo '|ticked=', count($ticked), ':', $ticked[0]['Name'];

$stmt = $wpdb->get_status_in_with_statement('wp_status_in_other', 'wp_status_in_alpha');
echo '|stmt=', count($stmt), ':';
foreach ($stmt as $row) {
    echo $row['Name'], ';';
}
