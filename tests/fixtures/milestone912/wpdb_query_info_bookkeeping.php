<?php
class wpdb {
    public $dbh;
    public $result;
    public $last_query;
    public $last_info;
    public $info_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->last_info = "unset";
        $this->info_checked = false;
    }

    public function _do_query($query) {
        $this->result = mysqli_query($this->dbh, $query);
    }

    public function query($query) {
        $this->last_query = $query;
        $this->_do_query($query);
        $this->last_info = mysqli_info($this->dbh);
        $this->info_checked = true;

        return $this->last_info === null ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->query("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "\n";
echo $wpdb->last_query;
echo "\n";
echo $wpdb->last_info === null ? "clean" : $wpdb->last_info;
echo "\n";
echo $wpdb->info_checked ? "checked" : "skipped";
