<?php
$stmt = mysqli_init();
mysqli_stmt_prepare($stmt, "SELECT option_value FROM wp_options WHERE option_name = ?");
