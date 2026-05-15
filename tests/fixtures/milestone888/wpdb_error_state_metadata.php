<?php
class wpdb {
    public $dbh;
    public $last_errno;
    public $last_error;
    public $last_sqlstate;
    public $warning_count;
    public $error_state_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_errno = -1;
        $this->last_error = "unread";
        $this->last_sqlstate = "";
        $this->warning_count = -1;
        $this->error_state_checked = false;
    }

    public function record_error_state() {
        $this->last_errno = mysqli_errno($this->dbh);
        $this->last_error = mysqli_error($this->dbh);
        $this->last_sqlstate = mysqli_sqlstate($this->dbh);
        $this->warning_count = mysqli_warning_count($this->dbh);
        $this->error_state_checked = true;

        return $this->last_sqlstate;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_error_state();
echo "\n";
echo $wpdb->last_errno;
echo "\n";
echo $wpdb->last_error;
echo "\n";
echo $wpdb->last_sqlstate;
echo "\n";
echo $wpdb->warning_count;
echo "\n";
echo $wpdb->error_state_checked ? "checked" : "skipped";
