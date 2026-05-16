<?php
class wpdb {
    public $dbh;
    public $last_query;
    public $multi_query_result;
    public $more_results;
    public $next_result;
    public $stored_result;
    public $used_result;
    public $query_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->multi_query_result = false;
        $this->more_results = true;
        $this->next_result = true;
        $this->stored_result = "unset";
        $this->used_result = "unset";
        $this->query_checked = false;
    }

    public function set_charset_with_multi_query($query) {
        $this->last_query = $query;
        $this->multi_query_result = mysqli_multi_query($this->dbh, $query);
        $this->more_results = mysqli_more_results($this->dbh);
        $this->next_result = mysqli_next_result($this->dbh);
        $this->stored_result = mysqli_store_result($this->dbh);
        $this->used_result = mysqli_use_result($this->dbh);
        $this->query_checked = true;

        return $this->multi_query_result
            && $this->more_results === false
            && $this->next_result === false
            && $this->stored_result === false
            && $this->used_result === false ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->set_charset_with_multi_query("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "\n";
echo $wpdb->last_query;
echo "\n";
echo $wpdb->multi_query_result ? "multi-query-ok" : "multi-query-failed";
echo "\n";
echo $wpdb->more_results ? "more" : "done";
echo "\n";
echo $wpdb->next_result ? "next" : "done";
echo "\n";
echo $wpdb->stored_result === false ? "no-pending" : "pending";
echo "\n";
echo $wpdb->used_result === false ? "no-use" : "using";
echo "\n";
echo $wpdb->query_checked ? "checked" : "skipped";
