<?php
$stmt = mysqli_init();
mysqli_stmt_send_long_data($stmt, 0, "blob");
