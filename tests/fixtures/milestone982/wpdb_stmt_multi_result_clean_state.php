<?php
class wpdb {
    public $dbh;
    public $more;
    public $next;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function record_multi_result_state() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_stmt_execute($stmt);
        $this->more = mysqli_stmt_more_results($stmt);
        $this->next = mysqli_stmt_next_result($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->record_multi_result_state();
echo $wpdb->more ? "more" : "no-more";
echo "|";
echo $wpdb->next ? "next" : "no-next";
