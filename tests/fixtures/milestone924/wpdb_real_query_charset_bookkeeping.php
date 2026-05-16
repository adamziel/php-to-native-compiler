<?php
class wpdb {
    public $dbh;
    public $last_query;
    public $real_query_result;
    public $stored_result;
    public $used_result;
    public $query_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->real_query_result = false;
        $this->stored_result = "unset";
        $this->used_result = "unset";
        $this->query_checked = false;
    }

    public function set_charset_with_real_query($query) {
        $this->last_query = $query;
        $this->real_query_result = mysqli_real_query($this->dbh, $query);
        $this->stored_result = mysqli_store_result($this->dbh);
        $this->used_result = mysqli_use_result($this->dbh);
        $this->query_checked = true;

        return $this->real_query_result && $this->stored_result === false && $this->used_result === false ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->set_charset_with_real_query("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "\n";
echo $wpdb->last_query;
echo "\n";
echo $wpdb->real_query_result ? "real-query-ok" : "real-query-failed";
echo "\n";
echo $wpdb->stored_result === false ? "no-pending" : "pending";
echo "\n";
echo $wpdb->used_result === false ? "no-use" : "using";
echo "\n";
echo $wpdb->query_checked ? "checked" : "skipped";
