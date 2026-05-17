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
        return mysqli_query($this->dbh, $query);
    }
}

function collect_columns($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Field'] . ':' . $row['Type'] . ':' . $row['Null'] . ':' . $row['Key'];
    }
    return implode(',', $parts);
}

function collect_indexes($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Key_name'] . ':' . $row['Column_name'] . ':' . $row['Sub_part'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_terms (term_id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', payload longtext NOT NULL, checksum varchar(64) NOT NULL default '', PRIMARY KEY  (term_id), KEY slug (slug), KEY checksum (checksum)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_probe_terms CHANGE COLUMN slug name varchar(200) NOT NULL default '', MODIFY COLUMN payload longtext NULL, DROP COLUMN checksum, DROP KEY slug, ADD KEY name (name(191))");

echo 'affected=' . mysqli_affected_rows($wpdb->dbh);
echo '|';

$columns = $wpdb->get_results('DESCRIBE wp_probe_terms');
echo 'columns=' . collect_columns($columns);
echo '|';

$indexes = $wpdb->get_results('SHOW INDEX FROM `wp_probe_terms`');
echo 'indexes=' . mysqli_num_rows($indexes) . ':' . collect_indexes($indexes);
