<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function stream_prepared_option_blob() {
        $stmt = $this->dbh;
        return mysqli_stmt_send_long_data($stmt, 0, "blob");
    }
}

$wpdb = new wpdb();
$wpdb->stream_prepared_option_blob();
