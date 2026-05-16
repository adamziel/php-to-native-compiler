<?php
$stmt = mysqli_init();
$value = "home";
mysqli_stmt_bind_param($stmt, "s", $value);
