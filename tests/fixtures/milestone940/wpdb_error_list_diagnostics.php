<?php
class wpdb {
    public $dbh;
    public $last_error_list;
    public $diagnostics_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_error_list = [];
        $this->diagnostics_checked = false;
    }

    public function record_error_list() {
        $this->last_error_list = mysqli_error_list($this->dbh);
        $this->diagnostics_checked = true;

        return count($this->last_error_list);
    }
}

$wpdb = new wpdb();
echo $wpdb->record_error_list();
echo "\n";
echo count($wpdb->last_error_list);
echo "\n";
echo mysqli_errno($wpdb->dbh);
echo "\n";
echo mysqli_error($wpdb->dbh);
echo "\n";
echo $wpdb->diagnostics_checked ? "checked" : "skipped";
