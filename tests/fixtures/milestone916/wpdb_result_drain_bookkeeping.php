<?php
class wpdb {
    public $dbh;
    public $last_query;
    public $stored_result;
    public $used_result;
    public $drain_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->stored_result = "unset";
        $this->used_result = "unset";
        $this->drain_checked = false;
    }

    public function drain_connection_results($query) {
        $this->last_query = $query;
        mysqli_query($this->dbh, $query);
        $this->stored_result = mysqli_store_result($this->dbh);
        $this->used_result = mysqli_use_result($this->dbh);
        $this->drain_checked = true;

        return $this->stored_result === false && $this->used_result === false ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->drain_connection_results("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "\n";
echo $wpdb->last_query;
echo "\n";
echo $wpdb->stored_result === false ? "no-store" : "stored";
echo "\n";
echo $wpdb->used_result === false ? "no-use" : "using";
echo "\n";
echo $wpdb->drain_checked ? "checked" : "skipped";
