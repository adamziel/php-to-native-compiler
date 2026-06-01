<?php
$code = '<?php
X::continue;
class C { const ARRAY = 1; use A { namespace as bar; } }
';
function dump_selected_tokens($label, $tokens) {
    echo "--", $label, "\n";
    foreach ($tokens as $token) {
        if (is_array($token)) {
            $name = token_name($token[0]);
            if ($token[1] == "continue" || $token[1] == "ARRAY" || $token[1] == "namespace") {
                echo $name, ":", $token[1], "\n";
            }
        }
    }
}
dump_selected_tokens("plain", token_get_all($code));
dump_selected_tokens("parse", token_get_all($code, TOKEN_PARSE));
