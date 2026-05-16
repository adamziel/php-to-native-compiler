<?php
class wpdb {
    public $dbh;
    public $sent;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function send_option_blob($value) {
        $stmt = mysqli_prepare($this->dbh, "SELECT option_value FROM wp_options WHERE option_name = ?");
        $this->sent = mysqli_stmt_send_long_data($stmt, 0, $value);
    }
}

$wpdb = new wpdb();
$wpdb->send_option_blob("blob");
echo $wpdb->sent ? "sent" : "failed";
