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

    public function get_tables($pattern) {
        $stmt = mysqli_prepare($this->dbh, 'SHOW TABLES LIKE ?');
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
$wpdb->query("CREATE TABLE wp_prepared_schema_probe (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, post_content longtext NOT NULL, PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10)), FULLTEXT KEY content_search (post_content)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$like = $wpdb->get_results('SHOW INDEX FROM wp_prepared_schema_probe WHERE Key_name LIKE ?', array('meta_%'));
echo 'like=', count($like), ':';
foreach ($like as $index) {
    echo $index['Key_name'], ':', $index['Column_name'], ';';
}

$escaped = $wpdb->get_results("SHOW INDEX FROM wp_prepared_schema_probe WHERE `Key_name` LIKE ? ESCAPE '!'", array('meta!_%'));
echo '|escaped=', count($escaped), ':';
foreach ($escaped as $index) {
    echo $index['Key_name'], ':', $index['Sub_part'], ';';
}

$columns = $wpdb->get_results('SHOW FULL COLUMNS FROM wp_prepared_schema_probe LIKE ?', array('meta_%'));
echo '|columns=', count($columns), ':';
foreach ($columns as $column) {
    echo $column['Field'], ':', $column['Key'], ';';
}

$field = $wpdb->get_results('SHOW COLUMNS FROM wp_prepared_schema_probe WHERE Field = ?', array('post_content'));
echo '|field=', count($field), ':', $field[0]['Field'], ':', $field[0]['Type'];

$key = $wpdb->get_results('SHOW INDEX FROM wp_prepared_schema_probe WHERE Key_name = ?', array('content_search'));
echo '|key=', count($key), ':', $key[0]['Key_name'], ':', $key[0]['Index_type'];

$status = $wpdb->get_results('SHOW TABLE STATUS LIKE ?', array('wp_prepared_schema_probe'));
echo '|status=', count($status), ':', $status[0]['Collation'];

$tables = $wpdb->get_tables('wp_prepared_schema_%');
echo '|tables=', count($tables), ':';
echo $tables[0]['Tables_in_wordpress (wp_prepared_schema_%)'];

$wpdb->query("SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = $wpdb->get_results('SHOW INDEX FROM wp_prepared_schema_probe WHERE Key_name LIKE ?', array('meta\\_%'));
echo '|mode=', count($mode);

$explicit = $wpdb->get_results("SHOW INDEX FROM wp_prepared_schema_probe WHERE Key_name LIKE ? ESCAPE '!'", array('meta!_%'));
echo '|mode-explicit=', count($explicit);
