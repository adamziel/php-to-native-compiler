use crate::ast::{Expr, Program, Statement};
use crate::diagnostic::{Diagnostic, Result, SourceSpan};
use crate::lexer::{lex, Token, TokenKind};

pub fn parse(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    Parser { tokens, index: 0 }.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program> {
        if matches!(self.peek().kind, TokenKind::OpenTag) {
            self.advance();
        }
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().kind {
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Variable(_) => self.parse_assignment(),
            _ => Err(Diagnostic::new(
                "expected statement",
                Some(self.peek().span),
            )),
        }
    }

    fn parse_assignment(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected variable", Some(token.span)));
        };
        self.expect_equal()?;
        let value = self.parse_expr()?;
        self.expect_semicolon()?;
        Ok(Statement::Assign {
            name,
            value,
            span: token.span,
        })
    }

    fn parse_echo(&mut self) -> Result<Statement> {
        let span = self.expect_echo()?;
        let mut expressions = vec![self.parse_expr()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            expressions.push(self.parse_expr()?);
        }
        self.expect_semicolon()?;
        Ok(Statement::Echo { expressions, span })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::String(value) => Ok(Expr::String(value, token.span)),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            _ => Err(Diagnostic::new("expected expression", Some(token.span))),
        }
    }

    fn expect_echo(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Echo) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected echo", Some(token.span)))
        }
    }

    fn expect_equal(&mut self) -> Result<()> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Equal) {
            Ok(())
        } else {
            Err(Diagnostic::new("expected assignment", Some(token.span)))
        }
    }

    fn expect_semicolon(&mut self) -> Result<()> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Semicolon) {
            Ok(())
        } else {
            Err(Diagnostic::new("expected semicolon", Some(token.span)))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.index];
        self.index += 1;
        token
    }
}
