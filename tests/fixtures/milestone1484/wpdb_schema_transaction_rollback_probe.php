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

$wpdb->query("CREATE TABLE wp_schema_base (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_begin_transaction($wpdb->dbh);
$wpdb->query("ALTER TABLE wp_schema_base ADD COLUMN checksum varchar(64) NOT NULL default '', ADD KEY checksum (checksum)");
$wpdb->query("CREATE TABLE wp_schema_temp (id bigint(20) unsigned NOT NULL, PRIMARY KEY  (id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
echo mysqli_rollback($wpdb->dbh) ? 'rollback' : 'failed';

echo '|temp=';
$temp = mysqli_query($wpdb->dbh, "SHOW TABLE STATUS LIKE 'wp_schema_temp'");
echo mysqli_num_rows($temp);

echo '|base=';
$base = $wpdb->get_results('DESCRIBE wp_schema_base');
foreach ($base as $column) {
    echo $column['Field'], ':', $column['Key'], ';';
}

mysqli_begin_transaction($wpdb->dbh);
mysqli_savepoint($wpdb->dbh, 'before_extra');
$wpdb->query("ALTER TABLE wp_schema_base ADD COLUMN extra varchar(20) NULL");
echo '|savepoint=';
echo mysqli_rollback($wpdb->dbh, 0, 'before_extra') ? 'rollback' : 'failed';

echo '|extra=';
$extra = mysqli_query($wpdb->dbh, "SHOW FULL COLUMNS FROM wp_schema_base LIKE 'extra'");
echo mysqli_num_rows($extra);
mysqli_commit($wpdb->dbh);

mysqli_begin_transaction($wpdb->dbh);
$wpdb->query("ALTER TABLE wp_schema_base ADD COLUMN committed varchar(20) NULL");
echo '|';
echo mysqli_commit($wpdb->dbh) ? 'commit' : 'failed';

echo '|committed=';
$committed = mysqli_query($wpdb->dbh, "SHOW FULL COLUMNS FROM wp_schema_base LIKE 'committed'");
echo mysqli_num_rows($committed);
