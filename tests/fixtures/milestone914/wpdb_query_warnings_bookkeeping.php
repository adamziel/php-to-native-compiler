<?php
class wpdb {
    public $dbh;
    public $result;
    public $last_query;
    public $last_warning;
    public $warning_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->last_warning = "unset";
        $this->warning_checked = false;
    }

    public function _do_query($query) {
        $this->result = mysqli_query($this->dbh, $query);
    }

    public function record_query_warnings($query) {
        $this->last_query = $query;
        $this->_do_query($query);
        $this->last_warning = mysqli_get_warnings($this->dbh);
        $this->warning_checked = true;

        return $this->last_warning === false ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_query_warnings("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "\n";
echo $wpdb->last_query;
echo "\n";
echo $wpdb->last_warning === false ? "clean" : "warning";
echo "\n";
echo $wpdb->warning_checked ? "checked" : "skipped";
