<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function get_results($query) {
        return mysqli_query($this->dbh, $query);
    }
}

function collect_index_rows($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['Key_name'] . ':' . $row['Seq_in_index'] . ':' . $row['Column_name'] . ':' . $row['Non_unique'] . ':' . $row['Index_type'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

$indexes = $wpdb->get_results('SHOW INDEX FROM wp_options;');
echo 'index=' . mysqli_num_fields($indexes) . ':' . mysqli_num_rows($indexes) . ':' . collect_index_rows($indexes);
echo '|';

$keys = $wpdb->get_results('SHOW KEYS FROM `wp_options`');
$first = mysqli_fetch_assoc($keys);
$second = mysqli_fetch_assoc($keys);
echo 'keys=' . $first['Key_name'] . ':' . $first['Column_name'] . ':' . $first['Visible'];
echo ',' . $second['Key_name'] . ':' . $second['Column_name'] . ':' . $second['Visible'];
