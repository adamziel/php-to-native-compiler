#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhpTokenizerToken {
    Token {
        id: i64,
        text: Vec<u8>,
        line: i64,
        position: i64,
    },
    Symbol {
        text: Vec<u8>,
        line: i64,
        position: i64,
    },
}

impl PhpTokenizerToken {
    pub fn id(&self) -> i64 {
        match self {
            Self::Token { id, .. } => *id,
            Self::Symbol { text, .. } => text.first().copied().unwrap_or_default() as i64,
        }
    }

    pub fn text(&self) -> &[u8] {
        match self {
            Self::Token { text, .. } | Self::Symbol { text, .. } => text,
        }
    }

    pub fn line(&self) -> i64 {
        match self {
            Self::Token { line, .. } | Self::Symbol { line, .. } => *line,
        }
    }

    pub fn position(&self) -> i64 {
        match self {
            Self::Token { position, .. } | Self::Symbol { position, .. } => *position,
        }
    }

    pub fn is_token_array(&self) -> bool {
        matches!(self, Self::Token { .. })
    }
}

pub const T_LNUMBER: i64 = 260;
pub const T_DNUMBER: i64 = 261;
pub const T_STRING: i64 = 262;
pub const T_NAME_FULLY_QUALIFIED: i64 = 263;
pub const T_NAME_RELATIVE: i64 = 264;
pub const T_NAME_QUALIFIED: i64 = 265;
pub const T_VARIABLE: i64 = 266;
pub const T_INLINE_HTML: i64 = 267;
pub const T_ENCAPSED_AND_WHITESPACE: i64 = 268;
pub const T_CONSTANT_ENCAPSED_STRING: i64 = 269;
pub const T_STRING_VARNAME: i64 = 270;
pub const T_NUM_STRING: i64 = 271;
pub const T_INCLUDE: i64 = 272;
pub const T_INCLUDE_ONCE: i64 = 273;
pub const T_EVAL: i64 = 274;
pub const T_REQUIRE: i64 = 275;
pub const T_REQUIRE_ONCE: i64 = 276;
pub const T_LOGICAL_OR: i64 = 277;
pub const T_LOGICAL_XOR: i64 = 278;
pub const T_LOGICAL_AND: i64 = 279;
pub const T_PRINT: i64 = 280;
pub const T_YIELD: i64 = 281;
pub const T_YIELD_FROM: i64 = 282;
pub const T_INSTANCEOF: i64 = 283;
pub const T_NEW: i64 = 284;
pub const T_CLONE: i64 = 285;
pub const T_EXIT: i64 = 286;
pub const T_IF: i64 = 287;
pub const T_ELSEIF: i64 = 288;
pub const T_ELSE: i64 = 289;
pub const T_ENDIF: i64 = 290;
pub const T_ECHO: i64 = 291;
pub const T_DO: i64 = 292;
pub const T_WHILE: i64 = 293;
pub const T_ENDWHILE: i64 = 294;
pub const T_FOR: i64 = 295;
pub const T_ENDFOR: i64 = 296;
pub const T_FOREACH: i64 = 297;
pub const T_ENDFOREACH: i64 = 298;
pub const T_DECLARE: i64 = 299;
pub const T_ENDDECLARE: i64 = 300;
pub const T_AS: i64 = 301;
pub const T_SWITCH: i64 = 302;
pub const T_ENDSWITCH: i64 = 303;
pub const T_CASE: i64 = 304;
pub const T_DEFAULT: i64 = 305;
pub const T_MATCH: i64 = 306;
pub const T_BREAK: i64 = 307;
pub const T_CONTINUE: i64 = 308;
pub const T_GOTO: i64 = 309;
pub const T_FUNCTION: i64 = 310;
pub const T_FN: i64 = 311;
pub const T_CONST: i64 = 312;
pub const T_RETURN: i64 = 313;
pub const T_TRY: i64 = 314;
pub const T_CATCH: i64 = 315;
pub const T_FINALLY: i64 = 316;
pub const T_THROW: i64 = 317;
pub const T_USE: i64 = 318;
pub const T_INSTEADOF: i64 = 319;
pub const T_GLOBAL: i64 = 320;
pub const T_STATIC: i64 = 321;
pub const T_ABSTRACT: i64 = 322;
pub const T_FINAL: i64 = 323;
pub const T_PRIVATE: i64 = 324;
pub const T_PROTECTED: i64 = 325;
pub const T_PUBLIC: i64 = 326;
pub const T_READONLY: i64 = 327;
pub const T_VAR: i64 = 328;
pub const T_UNSET: i64 = 329;
pub const T_ISSET: i64 = 330;
pub const T_EMPTY: i64 = 331;
pub const T_HALT_COMPILER: i64 = 332;
pub const T_CLASS: i64 = 333;
pub const T_TRAIT: i64 = 334;
pub const T_INTERFACE: i64 = 335;
pub const T_ENUM: i64 = 336;
pub const T_EXTENDS: i64 = 337;
pub const T_IMPLEMENTS: i64 = 338;
pub const T_NAMESPACE: i64 = 339;
pub const T_LIST: i64 = 340;
pub const T_ARRAY: i64 = 341;
pub const T_CALLABLE: i64 = 342;
pub const T_LINE: i64 = 343;
pub const T_FILE: i64 = 344;
pub const T_DIR: i64 = 345;
pub const T_CLASS_C: i64 = 346;
pub const T_TRAIT_C: i64 = 347;
pub const T_METHOD_C: i64 = 348;
pub const T_FUNC_C: i64 = 349;
pub const T_NS_C: i64 = 350;
pub const T_ATTRIBUTE: i64 = 351;
pub const T_PLUS_EQUAL: i64 = 352;
pub const T_MINUS_EQUAL: i64 = 353;
pub const T_MUL_EQUAL: i64 = 354;
pub const T_DIV_EQUAL: i64 = 355;
pub const T_CONCAT_EQUAL: i64 = 356;
pub const T_MOD_EQUAL: i64 = 357;
pub const T_AND_EQUAL: i64 = 358;
pub const T_OR_EQUAL: i64 = 359;
pub const T_XOR_EQUAL: i64 = 360;
pub const T_SL_EQUAL: i64 = 361;
pub const T_SR_EQUAL: i64 = 362;
pub const T_COALESCE_EQUAL: i64 = 363;
pub const T_BOOLEAN_OR: i64 = 364;
pub const T_BOOLEAN_AND: i64 = 365;
pub const T_IS_EQUAL: i64 = 366;
pub const T_IS_NOT_EQUAL: i64 = 367;
pub const T_IS_IDENTICAL: i64 = 368;
pub const T_IS_NOT_IDENTICAL: i64 = 369;
pub const T_IS_SMALLER_OR_EQUAL: i64 = 370;
pub const T_IS_GREATER_OR_EQUAL: i64 = 371;
pub const T_SPACESHIP: i64 = 372;
pub const T_SL: i64 = 373;
pub const T_SR: i64 = 374;
pub const T_INC: i64 = 375;
pub const T_DEC: i64 = 376;
pub const T_INT_CAST: i64 = 377;
pub const T_DOUBLE_CAST: i64 = 378;
pub const T_STRING_CAST: i64 = 379;
pub const T_ARRAY_CAST: i64 = 380;
pub const T_OBJECT_CAST: i64 = 381;
pub const T_BOOL_CAST: i64 = 382;
pub const T_UNSET_CAST: i64 = 383;
pub const T_OBJECT_OPERATOR: i64 = 384;
pub const T_NULLSAFE_OBJECT_OPERATOR: i64 = 385;
pub const T_DOUBLE_ARROW: i64 = 386;
pub const T_COMMENT: i64 = 387;
pub const T_DOC_COMMENT: i64 = 388;
pub const T_OPEN_TAG: i64 = 389;
pub const T_OPEN_TAG_WITH_ECHO: i64 = 390;
pub const T_CLOSE_TAG: i64 = 391;
pub const T_WHITESPACE: i64 = 392;
pub const T_START_HEREDOC: i64 = 393;
pub const T_END_HEREDOC: i64 = 394;
pub const T_DOLLAR_OPEN_CURLY_BRACES: i64 = 395;
pub const T_CURLY_OPEN: i64 = 396;
pub const T_PAAMAYIM_NEKUDOTAYIM: i64 = 397;
pub const T_NS_SEPARATOR: i64 = 398;
pub const T_ELLIPSIS: i64 = 399;
pub const T_COALESCE: i64 = 400;
pub const T_POW: i64 = 401;
pub const T_POW_EQUAL: i64 = 402;
pub const T_PROPERTY_C: i64 = 403;
pub const T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG: i64 = 404;
pub const T_BAD_CHARACTER: i64 = 405;
pub const T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG: i64 = 406;

pub fn token_id_by_constant_name(name: &str) -> Option<i64> {
    Some(match name {
        "T_LNUMBER" => T_LNUMBER,
        "T_DNUMBER" => T_DNUMBER,
        "T_STRING" => T_STRING,
        "T_NAME_FULLY_QUALIFIED" => T_NAME_FULLY_QUALIFIED,
        "T_NAME_RELATIVE" => T_NAME_RELATIVE,
        "T_NAME_QUALIFIED" => T_NAME_QUALIFIED,
        "T_VARIABLE" => T_VARIABLE,
        "T_INLINE_HTML" => T_INLINE_HTML,
        "T_ENCAPSED_AND_WHITESPACE" => T_ENCAPSED_AND_WHITESPACE,
        "T_CONSTANT_ENCAPSED_STRING" => T_CONSTANT_ENCAPSED_STRING,
        "T_STRING_VARNAME" => T_STRING_VARNAME,
        "T_NUM_STRING" => T_NUM_STRING,
        "T_INCLUDE" => T_INCLUDE,
        "T_INCLUDE_ONCE" => T_INCLUDE_ONCE,
        "T_EVAL" => T_EVAL,
        "T_REQUIRE" => T_REQUIRE,
        "T_REQUIRE_ONCE" => T_REQUIRE_ONCE,
        "T_LOGICAL_OR" => T_LOGICAL_OR,
        "T_LOGICAL_XOR" => T_LOGICAL_XOR,
        "T_LOGICAL_AND" => T_LOGICAL_AND,
        "T_PRINT" => T_PRINT,
        "T_YIELD" => T_YIELD,
        "T_YIELD_FROM" => T_YIELD_FROM,
        "T_INSTANCEOF" => T_INSTANCEOF,
        "T_NEW" => T_NEW,
        "T_CLONE" => T_CLONE,
        "T_EXIT" => T_EXIT,
        "T_IF" => T_IF,
        "T_ELSEIF" => T_ELSEIF,
        "T_ELSE" => T_ELSE,
        "T_ENDIF" => T_ENDIF,
        "T_ECHO" => T_ECHO,
        "T_DO" => T_DO,
        "T_WHILE" => T_WHILE,
        "T_ENDWHILE" => T_ENDWHILE,
        "T_FOR" => T_FOR,
        "T_ENDFOR" => T_ENDFOR,
        "T_FOREACH" => T_FOREACH,
        "T_ENDFOREACH" => T_ENDFOREACH,
        "T_DECLARE" => T_DECLARE,
        "T_ENDDECLARE" => T_ENDDECLARE,
        "T_AS" => T_AS,
        "T_SWITCH" => T_SWITCH,
        "T_ENDSWITCH" => T_ENDSWITCH,
        "T_CASE" => T_CASE,
        "T_DEFAULT" => T_DEFAULT,
        "T_MATCH" => T_MATCH,
        "T_BREAK" => T_BREAK,
        "T_CONTINUE" => T_CONTINUE,
        "T_GOTO" => T_GOTO,
        "T_FUNCTION" => T_FUNCTION,
        "T_FN" => T_FN,
        "T_CONST" => T_CONST,
        "T_RETURN" => T_RETURN,
        "T_TRY" => T_TRY,
        "T_CATCH" => T_CATCH,
        "T_FINALLY" => T_FINALLY,
        "T_THROW" => T_THROW,
        "T_USE" => T_USE,
        "T_INSTEADOF" => T_INSTEADOF,
        "T_GLOBAL" => T_GLOBAL,
        "T_STATIC" => T_STATIC,
        "T_ABSTRACT" => T_ABSTRACT,
        "T_FINAL" => T_FINAL,
        "T_PRIVATE" => T_PRIVATE,
        "T_PROTECTED" => T_PROTECTED,
        "T_PUBLIC" => T_PUBLIC,
        "T_READONLY" => T_READONLY,
        "T_VAR" => T_VAR,
        "T_UNSET" => T_UNSET,
        "T_ISSET" => T_ISSET,
        "T_EMPTY" => T_EMPTY,
        "T_HALT_COMPILER" => T_HALT_COMPILER,
        "T_CLASS" => T_CLASS,
        "T_TRAIT" => T_TRAIT,
        "T_INTERFACE" => T_INTERFACE,
        "T_ENUM" => T_ENUM,
        "T_EXTENDS" => T_EXTENDS,
        "T_IMPLEMENTS" => T_IMPLEMENTS,
        "T_NAMESPACE" => T_NAMESPACE,
        "T_LIST" => T_LIST,
        "T_ARRAY" => T_ARRAY,
        "T_CALLABLE" => T_CALLABLE,
        "T_LINE" => T_LINE,
        "T_FILE" => T_FILE,
        "T_DIR" => T_DIR,
        "T_CLASS_C" => T_CLASS_C,
        "T_TRAIT_C" => T_TRAIT_C,
        "T_METHOD_C" => T_METHOD_C,
        "T_FUNC_C" => T_FUNC_C,
        "T_NS_C" => T_NS_C,
        "T_ATTRIBUTE" => T_ATTRIBUTE,
        "T_PLUS_EQUAL" => T_PLUS_EQUAL,
        "T_MINUS_EQUAL" => T_MINUS_EQUAL,
        "T_MUL_EQUAL" => T_MUL_EQUAL,
        "T_DIV_EQUAL" => T_DIV_EQUAL,
        "T_CONCAT_EQUAL" => T_CONCAT_EQUAL,
        "T_MOD_EQUAL" => T_MOD_EQUAL,
        "T_AND_EQUAL" => T_AND_EQUAL,
        "T_OR_EQUAL" => T_OR_EQUAL,
        "T_XOR_EQUAL" => T_XOR_EQUAL,
        "T_SL_EQUAL" => T_SL_EQUAL,
        "T_SR_EQUAL" => T_SR_EQUAL,
        "T_COALESCE_EQUAL" => T_COALESCE_EQUAL,
        "T_BOOLEAN_OR" => T_BOOLEAN_OR,
        "T_BOOLEAN_AND" => T_BOOLEAN_AND,
        "T_IS_EQUAL" => T_IS_EQUAL,
        "T_IS_NOT_EQUAL" => T_IS_NOT_EQUAL,
        "T_IS_IDENTICAL" => T_IS_IDENTICAL,
        "T_IS_NOT_IDENTICAL" => T_IS_NOT_IDENTICAL,
        "T_IS_SMALLER_OR_EQUAL" => T_IS_SMALLER_OR_EQUAL,
        "T_IS_GREATER_OR_EQUAL" => T_IS_GREATER_OR_EQUAL,
        "T_SPACESHIP" => T_SPACESHIP,
        "T_SL" => T_SL,
        "T_SR" => T_SR,
        "T_INC" => T_INC,
        "T_DEC" => T_DEC,
        "T_INT_CAST" => T_INT_CAST,
        "T_DOUBLE_CAST" => T_DOUBLE_CAST,
        "T_STRING_CAST" => T_STRING_CAST,
        "T_ARRAY_CAST" => T_ARRAY_CAST,
        "T_OBJECT_CAST" => T_OBJECT_CAST,
        "T_BOOL_CAST" => T_BOOL_CAST,
        "T_UNSET_CAST" => T_UNSET_CAST,
        "T_OBJECT_OPERATOR" => T_OBJECT_OPERATOR,
        "T_NULLSAFE_OBJECT_OPERATOR" => T_NULLSAFE_OBJECT_OPERATOR,
        "T_DOUBLE_ARROW" => T_DOUBLE_ARROW,
        "T_COMMENT" => T_COMMENT,
        "T_DOC_COMMENT" => T_DOC_COMMENT,
        "T_OPEN_TAG" => T_OPEN_TAG,
        "T_OPEN_TAG_WITH_ECHO" => T_OPEN_TAG_WITH_ECHO,
        "T_CLOSE_TAG" => T_CLOSE_TAG,
        "T_WHITESPACE" => T_WHITESPACE,
        "T_START_HEREDOC" => T_START_HEREDOC,
        "T_END_HEREDOC" => T_END_HEREDOC,
        "T_DOLLAR_OPEN_CURLY_BRACES" => T_DOLLAR_OPEN_CURLY_BRACES,
        "T_CURLY_OPEN" => T_CURLY_OPEN,
        "T_PAAMAYIM_NEKUDOTAYIM" | "T_DOUBLE_COLON" => T_PAAMAYIM_NEKUDOTAYIM,
        "T_NS_SEPARATOR" => T_NS_SEPARATOR,
        "T_ELLIPSIS" => T_ELLIPSIS,
        "T_COALESCE" => T_COALESCE,
        "T_POW" => T_POW,
        "T_POW_EQUAL" => T_POW_EQUAL,
        "T_PROPERTY_C" => T_PROPERTY_C,
        "T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG" => T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG,
        "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG" => T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG,
        "T_BAD_CHARACTER" => T_BAD_CHARACTER,
        _ => return None,
    })
}

pub fn token_name(id: i64) -> &'static str {
    match id {
        T_LNUMBER => "T_LNUMBER",
        T_DNUMBER => "T_DNUMBER",
        T_STRING => "T_STRING",
        T_NAME_FULLY_QUALIFIED => "T_NAME_FULLY_QUALIFIED",
        T_NAME_RELATIVE => "T_NAME_RELATIVE",
        T_NAME_QUALIFIED => "T_NAME_QUALIFIED",
        T_VARIABLE => "T_VARIABLE",
        T_INLINE_HTML => "T_INLINE_HTML",
        T_ENCAPSED_AND_WHITESPACE => "T_ENCAPSED_AND_WHITESPACE",
        T_CONSTANT_ENCAPSED_STRING => "T_CONSTANT_ENCAPSED_STRING",
        T_STRING_VARNAME => "T_STRING_VARNAME",
        T_NUM_STRING => "T_NUM_STRING",
        T_INCLUDE => "T_INCLUDE",
        T_INCLUDE_ONCE => "T_INCLUDE_ONCE",
        T_EVAL => "T_EVAL",
        T_REQUIRE => "T_REQUIRE",
        T_REQUIRE_ONCE => "T_REQUIRE_ONCE",
        T_LOGICAL_OR => "T_LOGICAL_OR",
        T_LOGICAL_XOR => "T_LOGICAL_XOR",
        T_LOGICAL_AND => "T_LOGICAL_AND",
        T_PRINT => "T_PRINT",
        T_YIELD => "T_YIELD",
        T_YIELD_FROM => "T_YIELD_FROM",
        T_INSTANCEOF => "T_INSTANCEOF",
        T_NEW => "T_NEW",
        T_CLONE => "T_CLONE",
        T_EXIT => "T_EXIT",
        T_IF => "T_IF",
        T_ELSEIF => "T_ELSEIF",
        T_ELSE => "T_ELSE",
        T_ENDIF => "T_ENDIF",
        T_ECHO => "T_ECHO",
        T_DO => "T_DO",
        T_WHILE => "T_WHILE",
        T_ENDWHILE => "T_ENDWHILE",
        T_FOR => "T_FOR",
        T_ENDFOR => "T_ENDFOR",
        T_FOREACH => "T_FOREACH",
        T_ENDFOREACH => "T_ENDFOREACH",
        T_DECLARE => "T_DECLARE",
        T_ENDDECLARE => "T_ENDDECLARE",
        T_AS => "T_AS",
        T_SWITCH => "T_SWITCH",
        T_ENDSWITCH => "T_ENDSWITCH",
        T_CASE => "T_CASE",
        T_DEFAULT => "T_DEFAULT",
        T_MATCH => "T_MATCH",
        T_BREAK => "T_BREAK",
        T_CONTINUE => "T_CONTINUE",
        T_GOTO => "T_GOTO",
        T_FUNCTION => "T_FUNCTION",
        T_FN => "T_FN",
        T_CONST => "T_CONST",
        T_RETURN => "T_RETURN",
        T_TRY => "T_TRY",
        T_CATCH => "T_CATCH",
        T_FINALLY => "T_FINALLY",
        T_THROW => "T_THROW",
        T_USE => "T_USE",
        T_INSTEADOF => "T_INSTEADOF",
        T_GLOBAL => "T_GLOBAL",
        T_STATIC => "T_STATIC",
        T_ABSTRACT => "T_ABSTRACT",
        T_FINAL => "T_FINAL",
        T_PRIVATE => "T_PRIVATE",
        T_PROTECTED => "T_PROTECTED",
        T_PUBLIC => "T_PUBLIC",
        T_READONLY => "T_READONLY",
        T_VAR => "T_VAR",
        T_UNSET => "T_UNSET",
        T_ISSET => "T_ISSET",
        T_EMPTY => "T_EMPTY",
        T_HALT_COMPILER => "T_HALT_COMPILER",
        T_CLASS => "T_CLASS",
        T_TRAIT => "T_TRAIT",
        T_INTERFACE => "T_INTERFACE",
        T_ENUM => "T_ENUM",
        T_EXTENDS => "T_EXTENDS",
        T_IMPLEMENTS => "T_IMPLEMENTS",
        T_NAMESPACE => "T_NAMESPACE",
        T_LIST => "T_LIST",
        T_ARRAY => "T_ARRAY",
        T_CALLABLE => "T_CALLABLE",
        T_LINE => "T_LINE",
        T_FILE => "T_FILE",
        T_DIR => "T_DIR",
        T_CLASS_C => "T_CLASS_C",
        T_TRAIT_C => "T_TRAIT_C",
        T_METHOD_C => "T_METHOD_C",
        T_FUNC_C => "T_FUNC_C",
        T_NS_C => "T_NS_C",
        T_ATTRIBUTE => "T_ATTRIBUTE",
        T_PLUS_EQUAL => "T_PLUS_EQUAL",
        T_MINUS_EQUAL => "T_MINUS_EQUAL",
        T_MUL_EQUAL => "T_MUL_EQUAL",
        T_DIV_EQUAL => "T_DIV_EQUAL",
        T_CONCAT_EQUAL => "T_CONCAT_EQUAL",
        T_MOD_EQUAL => "T_MOD_EQUAL",
        T_AND_EQUAL => "T_AND_EQUAL",
        T_OR_EQUAL => "T_OR_EQUAL",
        T_XOR_EQUAL => "T_XOR_EQUAL",
        T_SL_EQUAL => "T_SL_EQUAL",
        T_SR_EQUAL => "T_SR_EQUAL",
        T_COALESCE_EQUAL => "T_COALESCE_EQUAL",
        T_BOOLEAN_OR => "T_BOOLEAN_OR",
        T_BOOLEAN_AND => "T_BOOLEAN_AND",
        T_IS_EQUAL => "T_IS_EQUAL",
        T_IS_NOT_EQUAL => "T_IS_NOT_EQUAL",
        T_IS_IDENTICAL => "T_IS_IDENTICAL",
        T_IS_NOT_IDENTICAL => "T_IS_NOT_IDENTICAL",
        T_IS_SMALLER_OR_EQUAL => "T_IS_SMALLER_OR_EQUAL",
        T_IS_GREATER_OR_EQUAL => "T_IS_GREATER_OR_EQUAL",
        T_SPACESHIP => "T_SPACESHIP",
        T_SL => "T_SL",
        T_SR => "T_SR",
        T_INC => "T_INC",
        T_DEC => "T_DEC",
        T_INT_CAST => "T_INT_CAST",
        T_DOUBLE_CAST => "T_DOUBLE_CAST",
        T_STRING_CAST => "T_STRING_CAST",
        T_ARRAY_CAST => "T_ARRAY_CAST",
        T_OBJECT_CAST => "T_OBJECT_CAST",
        T_BOOL_CAST => "T_BOOL_CAST",
        T_UNSET_CAST => "T_UNSET_CAST",
        T_OBJECT_OPERATOR => "T_OBJECT_OPERATOR",
        T_NULLSAFE_OBJECT_OPERATOR => "T_NULLSAFE_OBJECT_OPERATOR",
        T_DOUBLE_ARROW => "T_DOUBLE_ARROW",
        T_COMMENT => "T_COMMENT",
        T_DOC_COMMENT => "T_DOC_COMMENT",
        T_OPEN_TAG => "T_OPEN_TAG",
        T_OPEN_TAG_WITH_ECHO => "T_OPEN_TAG_WITH_ECHO",
        T_CLOSE_TAG => "T_CLOSE_TAG",
        T_WHITESPACE => "T_WHITESPACE",
        T_START_HEREDOC => "T_START_HEREDOC",
        T_END_HEREDOC => "T_END_HEREDOC",
        T_DOLLAR_OPEN_CURLY_BRACES => "T_DOLLAR_OPEN_CURLY_BRACES",
        T_CURLY_OPEN => "T_CURLY_OPEN",
        T_PAAMAYIM_NEKUDOTAYIM => "T_DOUBLE_COLON",
        T_NS_SEPARATOR => "T_NS_SEPARATOR",
        T_ELLIPSIS => "T_ELLIPSIS",
        T_COALESCE => "T_COALESCE",
        T_POW => "T_POW",
        T_POW_EQUAL => "T_POW_EQUAL",
        T_PROPERTY_C => "T_PROPERTY_C",
        T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG => "T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG",
        T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG => "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG",
        T_BAD_CHARACTER => "T_BAD_CHARACTER",
        _ => "UNKNOWN",
    }
}

pub fn tokenize(source: &[u8]) -> Vec<PhpTokenizerToken> {
    let mut scanner = Scanner::new(source);
    scanner.scan();
    scanner.tokens
}

struct Scanner<'a> {
    source: &'a [u8],
    index: usize,
    line: i64,
    in_php: bool,
    last_significant_token: Option<i64>,
    tokens: Vec<PhpTokenizerToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntegerBase {
    DecimalOrLegacyOctal,
    Hex,
    Binary,
    Octal,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            index: 0,
            line: 1,
            in_php: false,
            last_significant_token: None,
            tokens: Vec::new(),
        }
    }

    fn scan(&mut self) {
        while self.index < self.source.len() {
            if self.in_php {
                self.scan_php_token();
            } else {
                self.scan_inline_html();
            }
        }
    }

    fn scan_inline_html(&mut self) {
        let Some(tag_index) = self.next_open_tag_index() else {
            if self.index < self.source.len() {
                let start = self.index;
                self.index = self.source.len();
                self.push_token(T_INLINE_HTML, start, self.source.len());
            }
            return;
        };

        if tag_index > self.index {
            let start = self.index;
            self.index = tag_index;
            self.push_token(T_INLINE_HTML, start, tag_index);
            return;
        }

        self.consume_open_tag();
    }

    fn scan_php_token(&mut self) {
        if self.starts_with(b"?>") {
            let start = self.index;
            self.index += 2;
            if self.peek() == Some(b'\r') {
                self.index += 1;
                if self.peek() == Some(b'\n') {
                    self.index += 1;
                }
            } else if self.peek() == Some(b'\n') {
                self.index += 1;
            }
            self.push_token(T_CLOSE_TAG, start, self.index);
            self.in_php = false;
            return;
        }

        if let Some((id, len)) = self.cast_token_at_current() {
            let start = self.index;
            self.index += len;
            self.push_token(id, start, self.index);
            return;
        }

        let byte = self.source[self.index];
        if is_php_whitespace(byte) {
            let start = self.index;
            while self.peek().is_some_and(is_php_whitespace) {
                self.index += 1;
            }
            self.push_token(T_WHITESPACE, start, self.index);
            return;
        }

        if is_bad_character(byte) {
            let start = self.index;
            self.index += 1;
            self.push_token(T_BAD_CHARACTER, start, self.index);
            return;
        }

        if self.starts_with(b"/*") {
            self.consume_block_comment();
            return;
        }
        if self.starts_with(b"//") {
            self.consume_line_comment(T_COMMENT);
            return;
        }
        if byte == b'#' && !self.starts_with(b"#[") {
            self.consume_line_comment(T_COMMENT);
            return;
        }

        if byte == b'\'' || byte == b'"' {
            self.consume_quoted_string(byte);
            return;
        }

        if byte == b'$' {
            self.consume_variable_or_symbol();
            return;
        }

        if byte.is_ascii_digit() {
            self.consume_number();
            return;
        }

        if byte == b'.'
            && self
                .peek_offset(1)
                .is_some_and(|next| next.is_ascii_digit())
        {
            self.consume_leading_dot_number();
            return;
        }

        if byte == b'\\' {
            if self
                .peek_offset(1)
                .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
            {
                self.consume_qualified_name(T_NAME_FULLY_QUALIFIED);
            } else {
                let start = self.index;
                self.index += 1;
                self.push_token(T_NS_SEPARATOR, start, self.index);
            }
            return;
        }

        if byte == b'_' || byte.is_ascii_alphabetic() {
            self.consume_identifier_or_keyword();
            return;
        }

        if self.consume_operator_token() {
            return;
        }

        let start = self.index;
        self.index += 1;
        self.push_symbol(start, self.index);
    }

    fn next_open_tag_index(&self) -> Option<usize> {
        let mut cursor = self.index;
        while cursor < self.source.len() {
            if self.open_tag_at(cursor).is_some() {
                return Some(cursor);
            }
            cursor += 1;
        }
        None
    }

    fn open_tag_at(&self, index: usize) -> Option<usize> {
        if self.slice_eq_ignore_ascii_case(index, b"<?php") {
            let after = index + 5;
            if after == self.source.len()
                || self
                    .source
                    .get(after)
                    .copied()
                    .is_some_and(is_php_whitespace)
            {
                return Some(5);
            }
        }
        if self.source.get(index..index + 3) == Some(b"<?=") {
            return Some(3);
        }
        None
    }

    fn consume_open_tag(&mut self) {
        let start = self.index;
        if self.source.get(self.index..self.index + 3) == Some(b"<?=") {
            self.index += 3;
            self.push_token(T_OPEN_TAG_WITH_ECHO, start, self.index);
            self.in_php = true;
            return;
        }

        self.index += 5;
        if self.peek() == Some(b'\r') {
            self.index += 1;
            if self.peek() == Some(b'\n') {
                self.index += 1;
            }
        } else if self.peek().is_some_and(is_php_whitespace) {
            self.index += 1;
        }
        self.push_token(T_OPEN_TAG, start, self.index);
        self.in_php = true;
    }

    fn consume_block_comment(&mut self) {
        let start = self.index;
        let is_doc = self.starts_with(b"/**");
        self.index += 2;
        while self.index < self.source.len() && !self.starts_with(b"*/") {
            self.index += 1;
        }
        if self.starts_with(b"*/") {
            self.index += 2;
        }
        self.push_token(
            if is_doc { T_DOC_COMMENT } else { T_COMMENT },
            start,
            self.index,
        );
    }

    fn consume_line_comment(&mut self, id: i64) {
        let start = self.index;
        while let Some(byte) = self.peek() {
            if byte == b'\n' || byte == b'\r' {
                break;
            }
            self.index += 1;
        }
        self.push_token(id, start, self.index);
    }

    fn consume_quoted_string(&mut self, quote: u8) {
        if quote == b'\'' {
            let start = self.index;
            self.index += 1;
            while let Some(byte) = self.peek() {
                self.index += 1;
                if byte == b'\\' && self.index < self.source.len() {
                    self.index += 1;
                    continue;
                }
                if byte == quote {
                    break;
                }
            }
            self.push_token(T_CONSTANT_ENCAPSED_STRING, start, self.index);
            return;
        }

        let string_start = self.index;
        let mut cursor = self.index + 1;
        let mut contains_variable = false;
        while cursor < self.source.len() {
            let byte = self.source[cursor];
            if byte == b'\\' {
                cursor = cursor.saturating_add(2);
                continue;
            }
            if byte == b'"' {
                break;
            }
            if byte == b'$'
                && self
                    .source
                    .get(cursor + 1)
                    .copied()
                    .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
            {
                contains_variable = true;
            }
            cursor += 1;
        }
        let string_end = (cursor + usize::from(cursor < self.source.len())).min(self.source.len());
        if !contains_variable {
            self.index = string_end;
            self.push_token(T_CONSTANT_ENCAPSED_STRING, string_start, self.index);
            return;
        }

        let open_quote = self.index;
        self.index += 1;
        self.push_symbol(open_quote, open_quote + 1);
        let content_end = cursor;
        while self.index < content_end {
            if self.peek() == Some(b'$')
                && self
                    .peek_offset(1)
                    .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
            {
                self.consume_variable_or_symbol();
                continue;
            }
            let start = self.index;
            while self.index < content_end {
                if self.peek() == Some(b'$')
                    && self
                        .peek_offset(1)
                        .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
                {
                    break;
                }
                if self.peek() == Some(b'\\') && self.index + 1 < content_end {
                    self.index += 2;
                } else {
                    self.index += 1;
                }
            }
            if start < self.index {
                self.push_token(T_ENCAPSED_AND_WHITESPACE, start, self.index);
            }
        }
        if self.peek() == Some(b'"') {
            let close_quote = self.index;
            self.index += 1;
            self.push_symbol(close_quote, close_quote + 1);
        }
    }

    fn consume_variable_or_symbol(&mut self) {
        let start = self.index;
        self.index += 1;
        if self
            .peek()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        {
            self.index += 1;
            while self
                .peek()
                .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
            {
                self.index += 1;
            }
            self.push_token(T_VARIABLE, start, self.index);
        } else if self.peek() == Some(b'{') {
            self.index += 1;
            self.push_token(T_DOLLAR_OPEN_CURLY_BRACES, start, self.index);
        } else {
            self.push_symbol(start, self.index);
        }
    }

    fn consume_number(&mut self) {
        let start = self.index;

        if self.prefixed_number_at_current(b"0x", |byte| byte.is_ascii_hexdigit()) {
            self.index += 2;
            self.consume_digits_with_separators(|byte| byte.is_ascii_hexdigit());
            self.push_token(
                self.integer_token_id(start, self.index, IntegerBase::Hex),
                start,
                self.index,
            );
            return;
        }

        if self.prefixed_number_at_current(b"0b", |byte| matches!(byte, b'0' | b'1')) {
            self.index += 2;
            self.consume_digits_with_separators(|byte| matches!(byte, b'0' | b'1'));
            self.push_token(
                self.integer_token_id(start, self.index, IntegerBase::Binary),
                start,
                self.index,
            );
            return;
        }

        if self.prefixed_number_at_current(b"0o", |byte| matches!(byte, b'0'..=b'7')) {
            self.index += 2;
            self.consume_digits_with_separators(|byte| matches!(byte, b'0'..=b'7'));
            self.push_token(
                self.integer_token_id(start, self.index, IntegerBase::Octal),
                start,
                self.index,
            );
            return;
        }

        self.consume_digits_with_separators(|byte| byte.is_ascii_digit());

        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.index += 1;
            self.consume_digits_with_separators(|byte| byte.is_ascii_digit());
        }

        if self.consume_exponent_if_present() {
            is_float = true;
        }

        self.push_token(
            if is_float {
                T_DNUMBER
            } else {
                self.integer_token_id(start, self.index, IntegerBase::DecimalOrLegacyOctal)
            },
            start,
            self.index,
        );
    }

    fn consume_leading_dot_number(&mut self) {
        let start = self.index;
        self.index += 1;
        self.consume_digits_with_separators(|byte| byte.is_ascii_digit());
        self.consume_exponent_if_present();
        self.push_token(T_DNUMBER, start, self.index);
    }

    fn prefixed_number_at_current<F>(&self, prefix: &[u8], is_digit: F) -> bool
    where
        F: Fn(u8) -> bool,
    {
        self.slice_eq_ignore_ascii_case(self.index, prefix)
            && self
                .source
                .get(self.index + prefix.len())
                .copied()
                .is_some_and(is_digit)
    }

    fn consume_digits_with_separators<F>(&mut self, is_digit: F)
    where
        F: Copy + Fn(u8) -> bool,
    {
        while let Some(byte) = self.peek() {
            if is_digit(byte) {
                self.index += 1;
            } else if byte == b'_' && self.peek_offset(1).is_some_and(|next| is_digit(next)) {
                self.index += 1;
            } else {
                break;
            }
        }
    }

    fn consume_exponent_if_present(&mut self) -> bool {
        if !matches!(self.peek(), Some(b'e' | b'E')) {
            return false;
        }

        let sign_offset = usize::from(matches!(self.peek_offset(1), Some(b'+' | b'-')));
        if !self
            .peek_offset(1 + sign_offset)
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            return false;
        }

        self.index += 1 + sign_offset;
        self.consume_digits_with_separators(|byte| byte.is_ascii_digit());
        true
    }

    fn integer_token_id(&self, start: usize, end: usize, base: IntegerBase) -> i64 {
        if self.integer_literal_fits_i64(start, end, base) {
            T_LNUMBER
        } else {
            T_DNUMBER
        }
    }

    fn integer_literal_fits_i64(&self, start: usize, end: usize, base: IntegerBase) -> bool {
        let bytes = &self.source[start..end];
        match base {
            IntegerBase::Hex => integer_digits_fit_i64(&bytes[2..], 16, false),
            IntegerBase::Binary => integer_digits_fit_i64(&bytes[2..], 2, false),
            IntegerBase::Octal => integer_digits_fit_i64(&bytes[2..], 8, false),
            IntegerBase::DecimalOrLegacyOctal => {
                if bytes.len() > 1 && bytes.first() == Some(&b'0') {
                    integer_digits_fit_i64(bytes, 8, true)
                } else {
                    integer_digits_fit_i64(bytes, 10, false)
                }
            }
        }
    }

    fn consume_identifier_or_keyword(&mut self) {
        let start = self.index;
        self.index += 1;
        while self
            .peek()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        {
            self.index += 1;
        }

        let first_identifier_end = self.index;
        if self.peek() == Some(b'\\')
            && self
                .peek_offset(1)
                .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
        {
            while self.peek() == Some(b'\\')
                && self
                    .peek_offset(1)
                    .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
            {
                self.index += 2;
                while self
                    .peek()
                    .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
                {
                    self.index += 1;
                }
            }
            let id = if self.source[start..first_identifier_end].eq_ignore_ascii_case(b"namespace")
            {
                T_NAME_RELATIVE
            } else {
                T_NAME_QUALIFIED
            };
            self.push_token(id, start, self.index);
            return;
        }

        let id = if self.identifier_is_reserved_word_in_string_context() {
            T_STRING
        } else {
            token_id_for_identifier(&self.source[start..self.index]).unwrap_or(T_STRING)
        };
        self.push_token(id, start, self.index);
    }

    fn consume_qualified_name(&mut self, id: i64) {
        let start = self.index;
        self.index += 2;
        while self
            .peek()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        {
            self.index += 1;
        }
        while self.peek() == Some(b'\\')
            && self
                .peek_offset(1)
                .is_some_and(|next| next == b'_' || next.is_ascii_alphabetic())
        {
            self.index += 2;
            while self
                .peek()
                .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
            {
                self.index += 1;
            }
        }
        self.push_token(id, start, self.index);
    }

    fn consume_operator_token(&mut self) -> bool {
        for (pattern, id) in [
            (b"**=".as_slice(), T_POW_EQUAL),
            (b"<<=", T_SL_EQUAL),
            (b">>=", T_SR_EQUAL),
            (b"??=", T_COALESCE_EQUAL),
            (b"===", T_IS_IDENTICAL),
            (b"!==", T_IS_NOT_IDENTICAL),
            (b"<=>", T_SPACESHIP),
            (b"...", T_ELLIPSIS),
            (b"**", T_POW),
            (b"+=", T_PLUS_EQUAL),
            (b"-=", T_MINUS_EQUAL),
            (b"*=", T_MUL_EQUAL),
            (b"/=", T_DIV_EQUAL),
            (b".=", T_CONCAT_EQUAL),
            (b"%=", T_MOD_EQUAL),
            (b"&=", T_AND_EQUAL),
            (b"|=", T_OR_EQUAL),
            (b"^=", T_XOR_EQUAL),
            (b"||", T_BOOLEAN_OR),
            (b"&&", T_BOOLEAN_AND),
            (b"==", T_IS_EQUAL),
            (b"!=", T_IS_NOT_EQUAL),
            (b"<>", T_IS_NOT_EQUAL),
            (b"<=", T_IS_SMALLER_OR_EQUAL),
            (b">=", T_IS_GREATER_OR_EQUAL),
            (b"<<", T_SL),
            (b">>", T_SR),
            (b"++", T_INC),
            (b"--", T_DEC),
            (b"->", T_OBJECT_OPERATOR),
            (b"?->", T_NULLSAFE_OBJECT_OPERATOR),
            (b"=>", T_DOUBLE_ARROW),
            (b"::", T_PAAMAYIM_NEKUDOTAYIM),
            (b"#[", T_ATTRIBUTE),
            (b"??", T_COALESCE),
        ] {
            if self.starts_with(pattern) {
                let start = self.index;
                self.index += pattern.len();
                self.push_token(id, start, self.index);
                return true;
            }
        }
        if self.starts_with(b"&") {
            let start = self.index;
            self.index += 1;
            let id = if self.ampersand_is_followed_by_var_or_vararg() {
                T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG
            } else {
                T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG
            };
            self.push_token(id, start, self.index);
            return true;
        }
        false
    }

    fn identifier_is_reserved_word_in_string_context(&self) -> bool {
        matches!(
            self.last_significant_token,
            Some(
                T_OBJECT_OPERATOR
                    | T_NULLSAFE_OBJECT_OPERATOR
                    | T_PAAMAYIM_NEKUDOTAYIM
                    | T_CONST
                    | T_FUNCTION
                    | T_FN
                    | T_AS
                    | T_INSTEADOF
                    | T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG
                    | T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG
            )
        )
    }

    fn ampersand_is_followed_by_var_or_vararg(&self) -> bool {
        let mut cursor = self.index;
        while self
            .source
            .get(cursor)
            .copied()
            .is_some_and(is_php_whitespace)
        {
            cursor += 1;
        }

        self.source.get(cursor) == Some(&b'$')
            || self.source.get(cursor..cursor + 3) == Some(b"...")
    }

    fn cast_token_at_current(&self) -> Option<(i64, usize)> {
        let casts = [
            ("(int)", T_INT_CAST),
            ("(integer)", T_INT_CAST),
            ("(float)", T_DOUBLE_CAST),
            ("(double)", T_DOUBLE_CAST),
            ("(real)", T_DOUBLE_CAST),
            ("(string)", T_STRING_CAST),
            ("(binary)", T_STRING_CAST),
            ("(array)", T_ARRAY_CAST),
            ("(object)", T_OBJECT_CAST),
            ("(bool)", T_BOOL_CAST),
            ("(boolean)", T_BOOL_CAST),
            ("(unset)", T_UNSET_CAST),
        ];
        casts
            .iter()
            .find(|(cast, _)| self.slice_eq_ignore_ascii_case(self.index, cast.as_bytes()))
            .map(|(cast, id)| (*id, cast.len()))
    }

    fn push_token(&mut self, id: i64, start: usize, end: usize) {
        let text = self.source[start..end].to_vec();
        let line = self.line;
        let position = start as i64;
        self.line += byte_line_count(&text);
        self.note_significant_token(id);
        self.tokens.push(PhpTokenizerToken::Token {
            id,
            text,
            line,
            position,
        });
    }

    fn push_symbol(&mut self, start: usize, end: usize) {
        let text = self.source[start..end].to_vec();
        let line = self.line;
        let position = start as i64;
        self.line += byte_line_count(&text);
        if let Some(symbol) = text.first() {
            self.last_significant_token = Some(*symbol as i64);
        }
        self.tokens.push(PhpTokenizerToken::Symbol {
            text,
            line,
            position,
        });
    }

    fn starts_with(&self, pattern: &[u8]) -> bool {
        self.source
            .get(self.index..self.index + pattern.len())
            .is_some_and(|slice| slice == pattern)
    }

    fn slice_eq_ignore_ascii_case(&self, index: usize, pattern: &[u8]) -> bool {
        self.source
            .get(index..index + pattern.len())
            .is_some_and(|slice| slice.eq_ignore_ascii_case(pattern))
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.index).copied()
    }

    fn peek_offset(&self, offset: usize) -> Option<u8> {
        self.source.get(self.index + offset).copied()
    }

    fn note_significant_token(&mut self, id: i64) {
        if !matches!(
            id,
            T_INLINE_HTML
                | T_WHITESPACE
                | T_COMMENT
                | T_DOC_COMMENT
                | T_OPEN_TAG
                | T_CLOSE_TAG
                | T_OPEN_TAG_WITH_ECHO
        ) {
            self.last_significant_token = Some(id);
        }
    }
}

fn is_php_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\r' | b'\t' | 0x0b | 0x0c)
}

fn is_bad_character(byte: u8) -> bool {
    byte < 0x20 && !is_php_whitespace(byte)
}

fn byte_line_count(bytes: &[u8]) -> i64 {
    bytes.iter().filter(|byte| **byte == b'\n').count() as i64
}

fn integer_digits_fit_i64(bytes: &[u8], base: u32, legacy_octal_prefix: bool) -> bool {
    let mut value = 0u128;
    let limit = i64::MAX as u128;
    let mut saw_digit = false;

    for byte in bytes.iter().copied() {
        if byte == b'_' {
            continue;
        }

        let Some(digit) = integer_digit_value(byte) else {
            if legacy_octal_prefix {
                break;
            }
            return true;
        };
        if digit >= base {
            if legacy_octal_prefix {
                break;
            }
            return true;
        }

        saw_digit = true;
        value = value * base as u128 + digit as u128;
        if value > limit {
            return false;
        }
    }

    saw_digit
}

fn integer_digit_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
        b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
        _ => None,
    }
}

fn token_id_for_identifier(identifier: &[u8]) -> Option<i64> {
    let lower = identifier
        .iter()
        .map(u8::to_ascii_lowercase)
        .collect::<Vec<_>>();
    Some(match lower.as_slice() {
        b"include" => T_INCLUDE,
        b"include_once" => T_INCLUDE_ONCE,
        b"eval" => T_EVAL,
        b"require" => T_REQUIRE,
        b"require_once" => T_REQUIRE_ONCE,
        b"or" => T_LOGICAL_OR,
        b"xor" => T_LOGICAL_XOR,
        b"and" => T_LOGICAL_AND,
        b"print" => T_PRINT,
        b"yield" => T_YIELD,
        b"instanceof" => T_INSTANCEOF,
        b"new" => T_NEW,
        b"clone" => T_CLONE,
        b"exit" | b"die" => T_EXIT,
        b"if" => T_IF,
        b"elseif" => T_ELSEIF,
        b"else" => T_ELSE,
        b"endif" => T_ENDIF,
        b"echo" => T_ECHO,
        b"do" => T_DO,
        b"while" => T_WHILE,
        b"endwhile" => T_ENDWHILE,
        b"for" => T_FOR,
        b"endfor" => T_ENDFOR,
        b"foreach" => T_FOREACH,
        b"endforeach" => T_ENDFOREACH,
        b"declare" => T_DECLARE,
        b"enddeclare" => T_ENDDECLARE,
        b"as" => T_AS,
        b"switch" => T_SWITCH,
        b"endswitch" => T_ENDSWITCH,
        b"case" => T_CASE,
        b"default" => T_DEFAULT,
        b"match" => T_MATCH,
        b"break" => T_BREAK,
        b"continue" => T_CONTINUE,
        b"goto" => T_GOTO,
        b"function" => T_FUNCTION,
        b"fn" => T_FN,
        b"const" => T_CONST,
        b"return" => T_RETURN,
        b"try" => T_TRY,
        b"catch" => T_CATCH,
        b"finally" => T_FINALLY,
        b"throw" => T_THROW,
        b"use" => T_USE,
        b"insteadof" => T_INSTEADOF,
        b"global" => T_GLOBAL,
        b"static" => T_STATIC,
        b"abstract" => T_ABSTRACT,
        b"final" => T_FINAL,
        b"private" => T_PRIVATE,
        b"protected" => T_PROTECTED,
        b"public" => T_PUBLIC,
        b"readonly" => T_READONLY,
        b"var" => T_VAR,
        b"unset" => T_UNSET,
        b"isset" => T_ISSET,
        b"empty" => T_EMPTY,
        b"__halt_compiler" => T_HALT_COMPILER,
        b"class" => T_CLASS,
        b"trait" => T_TRAIT,
        b"interface" => T_INTERFACE,
        b"enum" => T_ENUM,
        b"extends" => T_EXTENDS,
        b"implements" => T_IMPLEMENTS,
        b"namespace" => T_NAMESPACE,
        b"list" => T_LIST,
        b"array" => T_ARRAY,
        b"callable" => T_CALLABLE,
        b"__line__" => T_LINE,
        b"__file__" => T_FILE,
        b"__dir__" => T_DIR,
        b"__class__" => T_CLASS_C,
        b"__trait__" => T_TRAIT_C,
        b"__method__" => T_METHOD_C,
        b"__function__" => T_FUNC_C,
        b"__namespace__" => T_NS_C,
        b"__property__" => T_PROPERTY_C,
        _ => return None,
    })
}
