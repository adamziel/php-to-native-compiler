use php_compiler::run_source;

#[test]
fn uri_whatwg_url_core_parses_normalizes_and_compares() {
    let execution = run_source(
        r#"<?php
$url = Uri\WhatWg\Url::parse("https://user:info@example.com:443/foo/bar?abc=123&def=ghi#hashmark");
echo $url->scheme, "|", $url->username, "|", $url->password, "|", $url->host, "|";
var_dump($url->port);
echo $url->path, "|", $url->query, "|", $url->fragment, "\n";
echo $url->toAsciiString(), "\n";
var_dump($url->getHostType());
var_dump($url->isSpecialScheme());
$same = new Uri\WhatWg\Url("HTTPS://user:info@EXAMPLE.COM:0443/../foo/bar?abc=123&def=ghi#hashmark");
var_dump($url->equals($same));
$frag = Uri\WhatWg\Url::parse("https://user:info@example.com/foo/bar?abc=123&def=ghi#other");
var_dump($url->equals($frag, Uri\UriComparisonMode::IncludeFragment));
var_dump($url->equals($frag, Uri\UriComparisonMode::ExcludeFragment));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "https|user|info|example.com|NULL\n/foo/bar|abc=123&def=ghi|hashmark\nhttps://user:info@example.com/foo/bar?abc=123&def=ghi#hashmark\nenum(Uri\\WhatWg\\UrlHostType::Domain)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uri_whatwg_url_core_handles_host_types_errors_and_relative_base() {
    let execution = run_source(
        r#"<?php
foreach ([
    "file:///E:/Documents%20and%20Settings",
    "mailto:user@example.com",
    "scheme://example.com",
    "https://192.168",
    "https://[2001:0db8:3333:4444:5555:6666:7777:8888]",
] as $input) {
    $url = Uri\WhatWg\Url::parse($input);
    echo $url->toAsciiString(), "|";
    var_dump($url->getHostType());
}
$base = Uri\WhatWg\Url::parse("https://example.com/path?query");
echo Uri\WhatWg\Url::parse("relative", $base)->toAsciiString(), "\n";
Uri\WhatWg\Url::parse("https://example.com", errors: $errors);
var_dump($errors);
foreach (["", "https://", "https://ex[a]mple.com", "https://[v7.host]"] as $input) {
    try {
        new Uri\WhatWg\Url($input);
    } catch (Throwable $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "file:///E:/Documents%20and%20Settings|enum(Uri\\WhatWg\\UrlHostType::Empty)\nmailto:user@example.com|NULL\nscheme://example.com|enum(Uri\\WhatWg\\UrlHostType::Opaque)\nhttps://192.0.0.168/|enum(Uri\\WhatWg\\UrlHostType::IPv4)\nhttps://[2001:db8:3333:4444:5555:6666:7777:8888]/|enum(Uri\\WhatWg\\UrlHostType::IPv6)\nhttps://example.com/relative\narray(0) {\n}\nUri\\WhatWg\\InvalidUrlException: The specified URI is malformed (MissingSchemeNonRelativeUrl)\nUri\\WhatWg\\InvalidUrlException: The specified URI is malformed (HostMissing)\nUri\\WhatWg\\InvalidUrlException: The specified URI is malformed (DomainInvalidCodePoint)\nUri\\WhatWg\\InvalidUrlException: The specified URI is malformed (Ipv6InvalidCodePoint)\n"
    );
    assert_eq!(execution.exit_code, 0);
}
