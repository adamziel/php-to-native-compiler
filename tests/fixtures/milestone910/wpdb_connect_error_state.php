<?php
class wpdb {
    public $dbh;
    public $last_errno;
    public $last_error;
    public $error_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_errno = -1;
        $this->last_error = "unset";
        $this->error_checked = false;
    }

    public function record_connect_error_state() {
        $this->last_errno = mysqli_connect_errno();
        $error = mysqli_connect_error();
        $this->last_error = $error === null ? "" : $error;
        $this->error_checked = true;

        return $this->last_errno;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_connect_error_state();
echo "\n";
echo $wpdb->last_errno;
echo "\n";
echo $wpdb->last_error === "" ? "clean" : $wpdb->last_error;
echo "\n";
echo $wpdb->error_checked ? "checked" : "skipped";
