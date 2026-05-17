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
$wpdb->query("CREATE TABLE wp_column_probe (inline_id bigint(20) unsigned NOT NULL auto_increment PRIMARY KEY, slug varchar(191) NOT NULL default 'draft\\'s', flag tinyint(1) NOT NULL default 0, maybe varchar(20) DEFAULT NULL, unique_code varchar(32) NOT NULL UNIQUE KEY, plain_key varchar(32) KEY) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_column_probe ADD COLUMN token varchar(20) NOT NULL default 'x' KEY, MODIFY COLUMN flag tinyint(1) NOT NULL default 1, CHANGE COLUMN slug post_slug varchar(191) NOT NULL default 'post' KEY");

$columns = $wpdb->get_results('DESCRIBE wp_column_probe');
foreach ($columns as $column) {
    echo $column['Field'], ':', $column['Type'], ':', $column['Null'], ':', $column['Key'], ':', $column['Default'], ':', $column['Extra'], ';';
}

$full = mysqli_query($wpdb->dbh, "SHOW FULL COLUMNS FROM wp_column_probe LIKE 'post_%'");
$full_row = mysqli_fetch_assoc($full);
echo '|';
echo $full_row['Field'], ':', $full_row['Collation'], ':', $full_row['Default'], ':', $full_row['Key'];

$indexes = $wpdb->get_results('SHOW INDEX FROM wp_column_probe');
echo '|';
foreach ($indexes as $index) {
    echo $index['Key_name'], ':', $index['Column_name'], ':', $index['Non_unique'], ';';
}

$created = mysqli_query($wpdb->dbh, 'SHOW CREATE TABLE wp_column_probe');
$created_row = mysqli_fetch_assoc($created);
echo '|';
echo $created_row['Create Table'];
