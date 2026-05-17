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

    public function get_results($query, $params = null) {
        if ($params === null) {
            $result = mysqli_query($this->dbh, $query);
        } else {
            $result = mysqli_execute_query($this->dbh, $query, $params);
        }
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }

    public function get_status_with_statement($table) {
        $stmt = mysqli_prepare($this->dbh, 'SHOW TABLE STATUS WHERE Name = ?');
        mysqli_stmt_execute($stmt, array($table));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

$wpdb = new wpdb();
$wpdb->query("CREATE TABLE wp_status_exact_probe (ID bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, PRIMARY KEY  (ID), KEY option_name (option_name)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");

$exact = $wpdb->get_results('SHOW TABLE STATUS WHERE Name = ?', array('wp_status_exact_probe'));
echo 'exact=', count($exact), ':', $exact[0]['Name'], ':', $exact[0]['Collation'];

$ticked = $wpdb->get_results('SHOW TABLE STATUS WHERE `Name` = ?', array('wp_status_exact_probe'));
echo '|ticked=', count($ticked), ':', $ticked[0]['Name'];

$missing = $wpdb->get_results('SHOW TABLE STATUS WHERE Name = ?', array('wp_status_missing_probe'));
echo '|missing=', count($missing);

$stmt = $wpdb->get_status_with_statement('wp_status_exact_probe');
echo '|stmt=', count($stmt), ':', $stmt[0]['Name'];
