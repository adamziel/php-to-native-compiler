use crate::ast::{ArrayItem, BinaryOp, Expr, FunctionDecl, Program, Span, Stmt, UnaryOp};
use crate::error::{CompileResult, Diagnostic, Phase};
use crate::lexer::{tokenize, Token, TokenKind};

pub fn parse_source(source: &str) -> CompileResult<Program> {
    let tokens = tokenize(source)?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_program(mut self) -> CompileResult<Program> {
        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::Eof)) {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> CompileResult<Stmt> {
        match &self.peek().kind {
            TokenKind::Function => self.parse_function(),
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Variable(_) if self.peek_next_is_equal() => self.parse_assignment(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_function(&mut self) -> CompileResult<Stmt> {
        let start = self
            .consume_keyword(TokenKind::Function, "expected 'function'")?
            .span;
        let name = self.consume_identifier("expected function name")?;
        self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            loop {
                let param = self.consume_variable("expected parameter name")?;
                params.push(param);
                if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
            }
        }

        self.consume_keyword(TokenKind::RParen, "expected ')' after parameter list")?;
        let body = self.parse_required_block("expected function body")?;

        Ok(Stmt::Function(FunctionDecl {
            name,
            params,
            body,
            span: start,
        }))
    }

    fn parse_echo(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Echo, "expected 'echo'")?
            .span;
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse_expression()?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after echo")?;
        Ok(Stmt::Echo { exprs, span })
    }

    fn parse_print(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Print, "expected 'print'")?
            .span;
        let expr = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after print")?;
        Ok(Stmt::Print { expr, span })
    }

    fn parse_if(&mut self) -> CompileResult<Stmt> {
        let span = self.consume_keyword(TokenKind::If, "expected 'if'")?.span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after if")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after if condition")?;
        let then_branch = self.parse_block_or_statement()?;
        let else_branch = if self.match_token(|kind| matches!(kind, TokenKind::Else)) {
            self.parse_block_or_statement()?
        } else {
            Vec::new()
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_while(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::While, "expected 'while'")?
            .span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after while")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after while condition")?;
        let body = self.parse_block_or_statement()?;
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_return(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Return, "expected 'return'")?
            .span;
        let value = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after return")?;
        Ok(Stmt::Return { value, span })
    }

    fn parse_assignment(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let (name, span) = match token.kind {
            TokenKind::Variable(name) => (name, token.span),
            _ => unreachable!("caller checks assignment start"),
        };
        self.consume_keyword(TokenKind::Equal, "expected '=' in assignment")?;
        let expr = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after assignment")?;
        Ok(Stmt::Assign { name, expr, span })
    }

    fn parse_expression_statement(&mut self) -> CompileResult<Stmt> {
        let expr = self.parse_expression()?;
        let span = expr.span();
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expr { expr, span })
    }

    fn parse_block_or_statement(&mut self) -> CompileResult<Vec<Stmt>> {
        if self.match_token(|kind| matches!(kind, TokenKind::LBrace)) {
            return self.parse_block_after_open();
        }

        Ok(vec![self.parse_statement()?])
    }

    fn parse_required_block(&mut self, message: &str) -> CompileResult<Vec<Stmt>> {
        self.consume_keyword(TokenKind::LBrace, message)?;
        self.parse_block_after_open()
    }

    fn parse_block_after_open(&mut self) -> CompileResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            statements.push(self.parse_statement()?);
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after block")?;
        Ok(statements)
    }

    fn parse_expression(&mut self) -> CompileResult<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::EqualEqual)) {
                BinaryOp::Eq
            } else if self.match_token(|kind| matches!(kind, TokenKind::BangEqual)) {
                BinaryOp::Ne
            } else {
                break;
            };
            let right = self.parse_comparison()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_concat()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Less)) {
                BinaryOp::Lt
            } else if self.match_token(|kind| matches!(kind, TokenKind::LessEqual)) {
                BinaryOp::Le
            } else if self.match_token(|kind| matches!(kind, TokenKind::Greater)) {
                BinaryOp::Gt
            } else if self.match_token(|kind| matches!(kind, TokenKind::GreaterEqual)) {
                BinaryOp::Ge
            } else {
                break;
            };
            let right = self.parse_concat()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_concat(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_additive()?;
        while self.match_token(|kind| matches!(kind, TokenKind::Dot)) {
            let right = self.parse_additive()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Concat,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Plus)) {
                BinaryOp::Add
            } else if self.match_token(|kind| matches!(kind, TokenKind::Minus)) {
                BinaryOp::Sub
            } else {
                break;
            };
            let right = self.parse_multiplicative()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Star)) {
                BinaryOp::Mul
            } else if self.match_token(|kind| matches!(kind, TokenKind::Slash)) {
                BinaryOp::Div
            } else {
                break;
            };
            let right = self.parse_unary()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> CompileResult<Expr> {
        if self.match_token(|kind| matches!(kind, TokenKind::Minus)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
                span,
            });
        }

        if self.match_token(|kind| matches!(kind, TokenKind::Bang)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> CompileResult<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::StringLiteral(value) => Ok(Expr::String(value, token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::LBracket => self.parse_array_literal(token.span),
            TokenKind::Identifier(name) => {
                self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;
                let mut args = Vec::new();
                if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                            break;
                        }
                    }
                }
                self.consume_keyword(TokenKind::RParen, "expected ')' after arguments")?;
                Ok(Expr::Call {
                    name,
                    args,
                    span: token.span,
                })
            }
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.consume_keyword(TokenKind::RParen, "expected ')' after expression")?;
                Ok(expr)
            }
            other => Err(self.error_at(
                token.span,
                format!("expected expression, found {}", token_name(&other)),
            )),
        }
    }

    fn parse_array_literal(&mut self, span: Span) -> CompileResult<Expr> {
        let mut items = Vec::new();
        if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
            return Ok(Expr::Array { items, span });
        }

        loop {
            let first = self.parse_expression()?;
            let item = if self.match_token(|kind| matches!(kind, TokenKind::FatArrow)) {
                ArrayItem {
                    key: Some(first),
                    value: self.parse_expression()?,
                }
            } else {
                ArrayItem {
                    key: None,
                    value: first,
                }
            };
            items.push(item);

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                break;
            }
        }

        self.consume_keyword(TokenKind::RBracket, "expected ']' after array literal")?;
        Ok(Expr::Array { items, span })
    }

    fn consume_identifier(&mut self, message: &str) -> CompileResult<String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok(name),
            _ => Err(self.error_at(token.span, message)),
        }
    }

    fn consume_variable(&mut self, message: &str) -> CompileResult<String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Variable(name) => Ok(name),
            _ => Err(self.error_at(token.span, message)),
        }
    }

    fn consume_keyword(&mut self, expected: TokenKind, message: &str) -> CompileResult<Token> {
        if same_variant(&self.peek().kind, &expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_at(self.peek().span, message))
        }
    }

    fn match_token(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        if predicate(&self.peek().kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        predicate(&self.peek().kind)
    }

    fn peek_next_is_equal(&self) -> bool {
        self.tokens
            .get(self.current + 1)
            .map(|token| matches!(token.kind, TokenKind::Equal))
            .unwrap_or(false)
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;
        if !matches!(self.tokens[index].kind, TokenKind::Eof) {
            self.current += 1;
        }
        &self.tokens[index]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error_at(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Parse, span.line, span.column, message)
    }
}

fn same_variant(left: &TokenKind, right: &TokenKind) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn token_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Eof => "end of file",
        TokenKind::Variable(_) => "variable",
        TokenKind::Identifier(_) => "identifier",
        TokenKind::Int(_) => "integer literal",
        TokenKind::Float(_) => "float literal",
        TokenKind::StringLiteral(_) => "string literal",
        TokenKind::Echo => "echo",
        TokenKind::Print => "print",
        TokenKind::Function => "function",
        TokenKind::Return => "return",
        TokenKind::If => "if",
        TokenKind::Else => "else",
        TokenKind::While => "while",
        TokenKind::Null => "null",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::Semicolon => ";",
        TokenKind::Comma => ",",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Dot => ".",
        TokenKind::Bang => "!",
        TokenKind::Equal => "=",
        TokenKind::FatArrow => "=>",
        TokenKind::EqualEqual => "==",
        TokenKind::BangEqual => "!=",
        TokenKind::Less => "<",
        TokenKind::LessEqual => "<=",
        TokenKind::Greater => ">",
        TokenKind::GreaterEqual => ">=",
    }
}
