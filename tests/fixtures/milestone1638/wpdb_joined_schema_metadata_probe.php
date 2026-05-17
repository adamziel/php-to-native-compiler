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

    public function get_results_with_statement($query, $table) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($table));
        $result = mysqli_stmt_get_result($stmt);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

function display_value($value) {
    return $value === null ? 'null' : $value;
}

$wpdb = new wpdb();
$wpdb->query("CREATE TABLE wp_joined_probe (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', locale varchar(20) NOT NULL default '', title text NULL, PRIMARY KEY  (id), KEY slug_locale (slug(64), locale)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$query = "SELECT c.COLUMN_NAME AS Field, c.COLUMN_TYPE AS Type, c.IS_NULLABLE AS `Null`, c.COLUMN_KEY AS `Key`, s.INDEX_NAME AS Key_name, s.SEQ_IN_INDEX AS Seq_in_index, s.SUB_PART AS Sub_part FROM information_schema.COLUMNS c LEFT JOIN information_schema.STATISTICS s ON s.TABLE_SCHEMA = c.TABLE_SCHEMA AND s.TABLE_NAME = c.TABLE_NAME AND s.COLUMN_NAME = c.COLUMN_NAME WHERE c.TABLE_SCHEMA = DATABASE() AND c.TABLE_NAME = ? ORDER BY c.ORDINAL_POSITION, s.SEQ_IN_INDEX";
$rows = $wpdb->get_results($query, array('wp_joined_probe'));
echo 'joined=', count($rows), ':';
foreach ($rows as $row) {
    echo $row['Field'], ':', $row['Key'], ':', display_value($row['Key_name']), ':', display_value($row['Seq_in_index']), ':', display_value($row['Sub_part']), ';';
}

$stmt_rows = $wpdb->get_results_with_statement($query, 'wp_joined_probe');
echo '|stmt=', count($stmt_rows), ':', $stmt_rows[1]['Field'], ':', $stmt_rows[1]['Key_name'];
