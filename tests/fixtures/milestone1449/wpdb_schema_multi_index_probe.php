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

function collect_indexes($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Key_name'] . ':' . $row['Seq_in_index'] . ':' . $row['Column_name'] . ':' . $row['Sub_part'];
    }
    return implode(',', $parts);
}

function collect_column_keys($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Field'] . ':' . $row['Key'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_posts (ID bigint(20) unsigned NOT NULL auto_increment, post_name varchar(200) NOT NULL default '', post_type varchar(20) NOT NULL default 'post', post_status varchar(20) NOT NULL default 'publish', post_date datetime NOT NULL default '0000-00-00 00:00:00', PRIMARY KEY  (ID), KEY type_status_date (post_type, post_status, post_date, ID), KEY post_name (post_name(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wpdb->query("ALTER TABLE wp_probe_posts ADD KEY name_date (post_name(191), post_date)");

$indexes = $wpdb->get_results('SHOW INDEX FROM `wp_probe_posts`');
echo 'indexes=' . mysqli_num_rows($indexes) . ':' . collect_indexes($indexes);
echo '|';

$columns = $wpdb->get_results('DESCRIBE wp_probe_posts');
echo 'columns=' . collect_column_keys($columns);
