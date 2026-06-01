<?php
$tokens = token_get_all('<?php
echo 1_000;
echo 0xCAFE_F00D;
echo 0b1010_0110;
echo 0o755;
echo .5e1;
echo 5.;
echo 5.e+1_2;
echo 9223372036854775808;
', TOKEN_PARSE);
foreach ($tokens as $token) {
    if (is_array($token)) {
        $name = token_name($token[0]);
        if ($name == "T_LNUMBER" || $name == "T_DNUMBER") {
            echo $name, ":", $token[1], "\n";
        }
    }
}
