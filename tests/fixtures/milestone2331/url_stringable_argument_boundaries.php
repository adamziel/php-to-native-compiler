<?php
class UrlText {
    public function __toString() { return "http://example.test/a b?x=1+2"; }
}
class EncodedText {
    public function __toString() { return "%41+%2B"; }
}
class PrefixText {
    public function __toString() { return "pre_"; }
}
class SeparatorText {
    public function __toString() { return "|"; }
}

var_dump(parse_url(new UrlText(), PHP_URL_HOST));
echo urlencode(new UrlText()), "\n";
echo rawurlencode(new UrlText()), "\n";
echo rawurldecode(new EncodedText()), "\n";
echo http_build_query([1], new PrefixText()), "\n";
echo http_build_query(["a" => 1, "b" => 2], "", new SeparatorText()), "\n";
try { parse_url(new stdClass()); } catch (Throwable $e) { echo "parse-object-caught\n"; }
try { urlencode([]); } catch (Throwable $e) { echo "urlencode-array-caught\n"; }
try { http_build_query([1], []); } catch (Throwable $e) { echo "prefix-array-caught\n"; }
try { http_build_query(["a" => 1], "", []); } catch (Throwable $e) { echo "separator-array-caught"; }
