use php_compiler::run_source;

#[test]
fn regex_iterator_exposes_regex_modes_flags_and_value_error() {
    let execution = run_source(
        r#"<?php
$it = new RegexIterator(new ArrayIterator(array("cat", "hat", "dog")), "/.at/");
echo $it->getRegex(), "|", $it->getMode(), "|", $it->getFlags(), "|", $it->getPregFlags(), "\n";
$it->setMode(RegexIterator::GET_MATCH);
$it->setFlags(RegexIterator::USE_KEY);
$it->setPregFlags(PREG_OFFSET_CAPTURE);
echo $it->getMode(), "|", $it->getFlags(), "|", $it->getPregFlags(), "\n";
try {
    $it->setMode(7);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/.at/|0|0|0\n1|1|256\nRegexIterator::setMode(): Argument #1 ($mode) must be RegexIterator::MATCH, RegexIterator::GET_MATCH, RegexIterator::ALL_MATCHES, RegexIterator::SPLIT, or RegexIterator::REPLACE\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn regex_iterator_filters_values_keys_and_inverted_matches() {
    let execution = run_source(
        r#"<?php
$ar = array(0, "123", 123, 22 => "abc", "a2b", 22, "a2d" => 7, 42);
foreach (new RegexIterator(new ArrayIterator($ar), "/2/") as $k => $v) {
    echo "$k=>$v\n";
}
echo "--keys--\n";
foreach (new RegexIterator(new ArrayIterator($ar), "/2/", RegexIterator::MATCH, RegexIterator::USE_KEY) as $k => $v) {
    echo "$k=>$v\n";
}
echo "--invert--\n";
foreach (new RegexIterator(new ArrayIterator(array("foo", "bar", "baz")), "/^ba/", RegexIterator::MATCH, RegexIterator::INVERT_MATCH) as $k => $v) {
    echo "$k=>$v\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1=>123\n2=>123\n23=>a2b\n24=>22\n25=>42\n--keys--\n2=>123\n22=>abc\n23=>a2b\n24=>22\na2d=>7\n25=>42\n--invert--\n0=>foo\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn regex_iterator_yields_match_split_and_replace_current_values() {
    let execution = run_source(
        r#"<?php
$ar = new ArrayIterator(array("1", "1,2", "1,2,3", "", NULL, array(), "FooBar", ",", ",,"));
$it = new RegexIterator($ar, "/(\d),(\d)/", RegexIterator::GET_MATCH);
foreach ($it as $k => $v) {
    echo "m:$k:", $v[0], "|", $v[1], "|", $v[2], "\n";
}
$it = new RegexIterator($ar, "/,/", RegexIterator::SPLIT);
foreach ($it as $k => $v) {
    echo "s:$k:", count($v), ":", $v[0], "|", $v[count($v) - 1], "\n";
}
$it = new RegexIterator(new ArrayIterator(array("test1" => "test888", "test2" => "what?", "test3" => "test999")), "/^test(.*)/", RegexIterator::REPLACE);
$it->replacement = "[$1]";
foreach ($it as $k => $v) {
    echo "r:$k=>$v\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "m:1:1,2|1|2\nm:2:1,2|1|2\ns:1:2:1|2\ns:2:3:1|3\ns:7:2:|\ns:8:3:|\nr:test1=>[888]\nr:test3=>[999]\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
