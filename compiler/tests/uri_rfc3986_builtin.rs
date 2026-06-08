use php_compiler::run_source;

#[test]
fn uri_rfc3986_preserves_bracketed_hosts_and_empty_port() {
    let execution = run_source(
        r#"<?php
$ipv6 = Uri\Rfc3986\Uri::parse("https://[2001:0db8:3333:4444:5555:6666:7777:8888]");
echo $ipv6->host, "|", $ipv6->toRawString(), "|", $ipv6->getHostType()->name, "\n";
$future = Uri\Rfc3986\Uri::parse("https://[vF.addr]");
echo $future->host, "|", $future->toRawString(), "|", $future->getHostType()->name, "\n";
$emptyPort = Uri\Rfc3986\Uri::parse("http://example.com:");
echo $emptyPort->host, "|";
var_dump($emptyPort->port);
echo $emptyPort->toRawString(), "\n";
$fragmentA = Uri\Rfc3986\Uri::parse("https://example.com/path#one");
$fragmentB = Uri\Rfc3986\Uri::parse("https://example.com/path#two");
var_dump($fragmentA->equals($fragmentB, Uri\UriComparisonMode::IncludeFragment));
var_dump($fragmentA->equals($fragmentB, Uri\UriComparisonMode::ExcludeFragment));
$relative = Uri\Rfc3986\Uri::parse("foo/bar");
var_dump($relative->getHostType());
var_dump($relative->getUriType());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "[2001:0db8:3333:4444:5555:6666:7777:8888]|https://[2001:0db8:3333:4444:5555:6666:7777:8888]|IPv6\n[vF.addr]|https://[vF.addr]|IPvFuture\nexample.com|NULL\nhttp://example.com:\nbool(false)\nbool(true)\nNULL\nenum(Uri\\Rfc3986\\UriType::RelativePathReference)\n"
    );
    assert_eq!(execution.exit_code, 0);
}
