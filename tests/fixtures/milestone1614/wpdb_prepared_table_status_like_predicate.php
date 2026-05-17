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

    public function get_status_like_with_statement($pattern) {
        $stmt = mysqli_prepare($this->dbh, "SHOW TABLE STATUS WHERE `Name` LIKE ? ESCAPE '!'");
        mysqli_stmt_execute($stmt, array($pattern));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb();
$wpdb->query("CREATE TABLE wp_status_like_probe (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$wpdb->query("CREATE TABLE wp_status_lake_probe (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(191) NOT NULL default '', PRIMARY KEY  (ID), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$like = $wpdb->get_results('SHOW TABLE STATUS WHERE Name LIKE ?', array('wp_status_l%_probe'));
echo 'like=', count($like), ':';
foreach ($like as $row) {
    echo $row['Name'], ':', $row['Collation'], ';';
}

$escaped = $wpdb->get_results("SHOW TABLE STATUS WHERE `Name` LIKE ? ESCAPE '!'", array('wp!_status!_like!_probe'));
echo '|escaped=', count($escaped), ':', $escaped[0]['Name'];

$stmt = $wpdb->get_status_like_with_statement('wp!_status!_l%');
echo '|stmt=', count($stmt), ':';
foreach ($stmt as $row) {
    echo $row['Name'], ';';
}
