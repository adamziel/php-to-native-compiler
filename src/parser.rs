use crate::ast::{AssignmentOp, BinaryOp, CastKind, Expr, Program, Statement, UnaryOp};
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
        self.expect_open_tag()?;
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::CloseTag | TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }
        if matches!(self.peek().kind, TokenKind::CloseTag) {
            self.advance();
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().kind {
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
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
        let op = self.expect_assignment_op()?;
        let value = self.parse_expr()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Assign {
            name,
            op,
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
        self.expect_statement_terminator()?;
        Ok(Statement::Echo { expressions, span })
    }

    fn parse_print(&mut self) -> Result<Statement> {
        let span = self.expect_print()?;
        let expression = self.parse_expr()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Print { expression, span })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr> {
        let mut left = self.parse_unary_expr()?;
        while let Some((op, precedence)) = self.peek_binary_op() {
            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_binary_expr(precedence + 1)?;
            let span = combine_spans(left.span(), right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr> {
        match self.peek().kind {
            TokenKind::Minus => {
                let token = self.advance().clone();
                let expr = self.parse_unary_expr()?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Bang => {
                let token = self.advance().clone();
                let expr = self.parse_unary_expr()?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LeftParen => {
                if let Some((kind, span)) = self.try_parse_cast_prefix()? {
                    let expr = self.parse_unary_expr()?;
                    let span = combine_spans(span, expr.span());
                    Ok(Expr::Cast {
                        kind,
                        expr: Box::new(expr),
                        span,
                    })
                } else {
                    self.parse_primary_expr()
                }
            }
            _ => self.parse_primary_expr(),
        }
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::String(value) => Ok(Expr::String(value, token.span)),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                self.expect_right_paren()?;
                Ok(expr)
            }
            _ => Err(Diagnostic::new("expected expression", Some(token.span))),
        }
    }

    fn try_parse_cast_prefix(&mut self) -> Result<Option<(CastKind, SourceSpan)>> {
        let start = self.index;
        let left = self.advance().clone();
        let Some(kind) = self.peek_cast_kind() else {
            self.index = start;
            return Ok(None);
        };
        self.advance();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            self.index = start;
            return Ok(None);
        }
        let right = self.advance().clone();
        Ok(Some((kind, combine_spans(left.span, right.span))))
    }

    fn peek_cast_kind(&self) -> Option<CastKind> {
        match self.peek().kind {
            TokenKind::IntType => Some(CastKind::Int),
            TokenKind::FloatType => Some(CastKind::Float),
            TokenKind::StringType => Some(CastKind::String),
            TokenKind::BoolType => Some(CastKind::Bool),
            _ => None,
        }
    }

    fn peek_binary_op(&self) -> Option<(BinaryOp, u8)> {
        match self.peek().kind {
            TokenKind::Dot => Some((BinaryOp::Concat, 10)),
            TokenKind::Plus => Some((BinaryOp::Add, 20)),
            _ => None,
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

    fn expect_print(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Print) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected print", Some(token.span)))
        }
    }

    fn expect_open_tag(&mut self) -> Result<()> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::OpenTag) {
            Ok(())
        } else {
            Err(Diagnostic::new("expected <?php open tag", Some(token.span)))
        }
    }

    fn expect_assignment_op(&mut self) -> Result<AssignmentOp> {
        let token = self.advance();
        match token.kind {
            TokenKind::Equal => Ok(AssignmentOp::Assign),
            TokenKind::PlusEqual => Ok(AssignmentOp::AddAssign),
            TokenKind::DotEqual => Ok(AssignmentOp::ConcatAssign),
            _ => Err(Diagnostic::new("expected assignment", Some(token.span))),
        }
    }

    fn expect_statement_terminator(&mut self) -> Result<()> {
        match self.peek().kind {
            TokenKind::Semicolon => {
                self.advance();
                Ok(())
            }
            TokenKind::CloseTag => Ok(()),
            _ => Err(Diagnostic::new(
                "expected semicolon",
                Some(self.peek().span),
            )),
        }
    }

    fn expect_right_paren(&mut self) -> Result<()> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightParen) {
            Ok(())
        } else {
            Err(Diagnostic::new(
                "expected right parenthesis",
                Some(token.span),
            ))
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

fn combine_spans(left: SourceSpan, right: SourceSpan) -> SourceSpan {
    SourceSpan::new(left.byte_start, right.byte_end, left.line, left.column)
}
