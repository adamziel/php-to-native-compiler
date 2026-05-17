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

    public function get_columns_with_statement($table) {
        $stmt = mysqli_prepare($this->dbh, 'SHOW FULL COLUMNS FROM ?');
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
$wpdb->query("CREATE TABLE wp_identifier_probe (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', title text NULL, PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$columns = $wpdb->get_results('SHOW FULL COLUMNS FROM ? WHERE Field = ?', array('wp_identifier_probe', 'slug'));
echo 'column=', count($columns), ':', $columns[0]['Field'], ':', $columns[0]['Key'], ':', $columns[0]['Collation'];

$indexes = $wpdb->get_results('SHOW INDEX FROM ? WHERE Key_name LIKE ?', array('wp_identifier_probe', 'slu%'));
echo '|index=', count($indexes), ':', $indexes[0]['Table'], ':', $indexes[0]['Key_name'], ':', $indexes[0]['Column_name'];

$all = $wpdb->get_columns_with_statement('wp_identifier_probe');
echo '|stmt=', count($all);
