<?php
class wpdb {
    public $dbh;
    public $stmt;
    public $last_errno;
    public $last_error;
    public $last_sqlstate;
    public $warning_count;
    public $error_count;
    public $rows_affected;
    public $insert_id;

    public function __construct() {
        $this->dbh = mysqli_init();
        $this->stmt = mysqli_stmt_init($this->dbh);
    }

    public function record_statement_diagnostics() {
        $this->last_errno = mysqli_stmt_errno($this->stmt);
        $this->last_error = mysqli_stmt_error($this->stmt);
        $this->last_sqlstate = mysqli_stmt_sqlstate($this->stmt);
        $this->warning_count = mysqli_stmt_warning_count($this->stmt);
        $this->error_count = count(mysqli_stmt_error_list($this->stmt));
        $this->rows_affected = mysqli_stmt_affected_rows($this->stmt);
        $this->insert_id = mysqli_stmt_insert_id($this->stmt);
        return mysqli_stmt_get_warnings($this->stmt);
    }
}

$wpdb = new wpdb();
$warnings = $wpdb->record_statement_diagnostics();
echo $wpdb->last_errno;
echo "|";
echo $wpdb->last_error === "" ? "empty" : "non-empty";
echo "|";
echo $wpdb->last_sqlstate;
echo "|";
echo $wpdb->warning_count;
echo "|";
echo $warnings === false ? "false" : "warning";
echo "|";
echo $wpdb->error_count;
echo "|";
echo $wpdb->rows_affected;
echo "|";
echo $wpdb->insert_id;
