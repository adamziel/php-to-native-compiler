<?php
class wpdb {
    public $dbh;
    public $num_rows;
    public $seeked;
    public $after_free;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function seek_statement_result() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_stmt_execute($stmt);
        mysqli_stmt_store_result($stmt);
        $this->num_rows = mysqli_stmt_num_rows($stmt);
        mysqli_stmt_data_seek($stmt, 0);
        $this->seeked = true;
        mysqli_stmt_free_result($stmt);
        $this->after_free = mysqli_stmt_num_rows($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->seek_statement_result();
echo $wpdb->num_rows;
echo "|";
echo $wpdb->seeked ? "seeked" : "not-seeked";
echo "|";
echo $wpdb->after_free;
