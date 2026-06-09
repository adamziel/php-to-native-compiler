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
        while !matches!(self.peek().kind, TokenKind::Eof) {
            if matches!(self.peek().kind, TokenKind::OpenTag | TokenKind::CloseTag) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().kind {
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::Identifier(_) => self.parse_call_statement(),
            TokenKind::Variable(_) => self.parse_assignment(),
            TokenKind::InlineHtml(_) => self.parse_inline_html(),
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

    fn parse_if(&mut self) -> Result<Statement> {
        let span = self.expect_if_like()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        let then_body = self.parse_block()?;
        let else_body = match self.peek().kind {
            TokenKind::Elseif => vec![self.parse_if()?],
            TokenKind::Else => {
                self.advance();
                self.parse_block()?
            }
            _ => Vec::new(),
        };
        Ok(Statement::If {
            condition,
            then_body,
            else_body,
            span,
        })
    }

    fn parse_call_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected function name", Some(token.span)));
        };
        self.expect_left_paren()?;
        let mut arguments = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            arguments.push(self.parse_expr()?);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                arguments.push(self.parse_expr()?);
            }
        }
        self.expect_right_paren()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Call {
            name: name.to_ascii_lowercase(),
            arguments,
            span: token.span,
        })
    }

    fn parse_inline_html(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::InlineHtml(content) = token.kind else {
            return Err(Diagnostic::new("expected inline HTML", Some(token.span)));
        };
        Ok(Statement::InlineHtml {
            content,
            span: token.span,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        self.expect_left_brace()?;
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }
        self.expect_right_brace()?;
        Ok(statements)
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
                let right_span = self.expect_right_paren()?;
                Ok(Expr::Grouped {
                    expr: Box::new(expr),
                    span: combine_spans(token.span, right_span),
                })
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
            TokenKind::OrOr => Some((BinaryOp::Or, 1)),
            TokenKind::AndAnd => Some((BinaryOp::And, 2)),
            TokenKind::EqualEqual => Some((BinaryOp::Equal, 5)),
            TokenKind::NotEqual => Some((BinaryOp::NotEqual, 5)),
            TokenKind::Less => Some((BinaryOp::Less, 6)),
            TokenKind::LessEqual => Some((BinaryOp::LessEqual, 6)),
            TokenKind::Greater => Some((BinaryOp::Greater, 6)),
            TokenKind::GreaterEqual => Some((BinaryOp::GreaterEqual, 6)),
            TokenKind::Dot => Some((BinaryOp::Concat, 10)),
            TokenKind::Plus => Some((BinaryOp::Add, 20)),
            TokenKind::Minus => Some((BinaryOp::Subtract, 20)),
            TokenKind::Asterisk => Some((BinaryOp::Multiply, 30)),
            TokenKind::Slash => Some((BinaryOp::Divide, 30)),
            TokenKind::Percent => Some((BinaryOp::Modulo, 30)),
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

    fn expect_if_like(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::If | TokenKind::Elseif) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected if", Some(token.span)))
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
            TokenKind::MinusEqual => Ok(AssignmentOp::SubtractAssign),
            TokenKind::AsteriskEqual => Ok(AssignmentOp::MultiplyAssign),
            TokenKind::SlashEqual => Ok(AssignmentOp::DivideAssign),
            TokenKind::PercentEqual => Ok(AssignmentOp::ModuloAssign),
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
            TokenKind::CloseTag | TokenKind::Eof => Ok(()),
            _ => Err(Diagnostic::new(
                "expected semicolon",
                Some(self.peek().span),
            )),
        }
    }

    fn expect_left_paren(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::LeftParen) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new(
                "expected left parenthesis",
                Some(token.span),
            ))
        }
    }

    fn expect_right_paren(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightParen) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new(
                "expected right parenthesis",
                Some(token.span),
            ))
        }
    }

    fn expect_left_brace(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::LeftBrace) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected left brace", Some(token.span)))
        }
    }

    fn expect_right_brace(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightBrace) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected right brace", Some(token.span)))
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
