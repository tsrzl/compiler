//! A recursive-descent parser for the syntax tree's initial statement forms.

mod modifiers;

use crate::source_text::Utf16Offset;
use crate::syntax::scanner::{Token, TokenKind};
use crate::syntax::{
    ArrowFunctionBody, AssignmentOperator, BinaryOperator, CatchClause, ClassDeclaration,
    ClassMember, Diagnostic, EnumDeclaration, EnumMember, ExportAllDeclaration,
    ExportNamedFromDeclaration, ExportSpecifier, Expression, ForInitializer, FunctionBodyStatement,
    FunctionDeclaration, FunctionParameter, ImportDeclaration, ImportSpecifier,
    InterfaceDeclaration, NamespaceDeclaration, ObjectProperty, Program, PropertyDeclaration,
    PropertySignature, ReturnStatement, Statement, SwitchClause, TextSpan, TypeAliasDeclaration,
    TypeReference, UnaryOperator, VariableDeclaration, VariableDeclarationKind,
};

pub(super) struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: Vec<Diagnostic>,
    loop_depth: usize,
    switch_depth: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostics: Vec::new(),
            loop_depth: 0,
            switch_depth: 0,
        }
    }

    pub(super) fn parse_program(mut self) -> (Program, Vec<Diagnostic>) {
        let mut statements = Vec::new();
        while !self.at_end() {
            match self.parse_statement() {
                Ok(statement) => {
                    let declaration_kind = variable_declaration_kind(&statement);
                    statements.push(statement);
                    if let Some((declaration_kind, exported)) = declaration_kind {
                        while self.matches(&TokenKind::Comma) {
                            match self.parse_variable_declarator(declaration_kind) {
                                Ok(declaration) => {
                                    let statement = Statement::VariableDeclaration(declaration);
                                    statements.push(if exported {
                                        Statement::ExportedDeclaration(Box::new(statement))
                                    } else {
                                        statement
                                    });
                                }
                                Err(diagnostic) => {
                                    self.diagnostics.push(diagnostic);
                                    self.synchronize();
                                    break;
                                }
                            }
                        }
                        self.matches(&TokenKind::Semicolon);
                    }
                }
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.synchronize();
                }
            }
        }

        (Program { statements }, self.diagnostics)
    }

    fn parse_statement(&mut self) -> Result<Statement, Diagnostic> {
        match self.peek().kind {
            TokenKind::Const
                if matches!(
                    self.tokens.get(self.current + 1).map(|token| &token.kind),
                    Some(TokenKind::Enum)
                ) =>
            {
                self.advance();
                self.parse_enum_declaration(true)
                    .map(Statement::EnumDeclaration)
            }
            TokenKind::Const | TokenKind::Let | TokenKind::Var => self
                .parse_variable_declaration()
                .map(Statement::VariableDeclaration),
            TokenKind::Interface => self
                .parse_interface_declaration()
                .map(Statement::InterfaceDeclaration),
            TokenKind::Enum => self
                .parse_enum_declaration(false)
                .map(Statement::EnumDeclaration),
            TokenKind::Namespace => self
                .parse_namespace_declaration()
                .map(Statement::NamespaceDeclaration),
            TokenKind::Type => self
                .parse_type_alias_declaration()
                .map(Statement::TypeAliasDeclaration),
            TokenKind::Function => self
                .parse_function_declaration()
                .map(Statement::FunctionDeclaration),
            TokenKind::Class => self
                .parse_class_declaration()
                .map(Statement::ClassDeclaration),
            TokenKind::Break => {
                let span = self.advance().span;
                self.matches(&TokenKind::Semicolon);
                Ok(Statement::Break { span })
            }
            TokenKind::Continue => {
                let span = self.advance().span;
                self.matches(&TokenKind::Semicolon);
                Ok(Statement::Continue { span })
            }
            TokenKind::Import => self
                .parse_import_declaration()
                .map(Statement::ImportDeclaration),
            TokenKind::Export => {
                self.advance();
                if self.matches(&TokenKind::Default) {
                    let expression = self.parse_expression()?;
                    self.matches(&TokenKind::Semicolon);
                    Ok(Statement::ExportDefault(expression))
                } else if matches!(self.peek().kind, TokenKind::Type)
                    && matches!(
                        self.tokens.get(self.current + 1).map(|token| &token.kind),
                        Some(TokenKind::LeftBrace)
                    )
                {
                    self.advance();
                    self.advance();
                    self.parse_named_export_declaration(true)
                } else if self.matches(&TokenKind::LeftBrace) {
                    self.parse_named_export_declaration(false)
                } else if self.matches(&TokenKind::Asterisk) {
                    self.parse_export_all_declaration()
                } else {
                    self.parse_statement()
                        .map(|statement| Statement::ExportedDeclaration(Box::new(statement)))
                }
            }
            _ => {
                if self.is_control_flow_start() {
                    return self.parse_top_level_control_flow_statement();
                }
                let expression = self.parse_expression()?;
                self.matches(&TokenKind::Semicolon);
                Ok(Statement::ExpressionStatement(expression))
            }
        }
    }

    fn parse_function_declaration(&mut self) -> Result<FunctionDeclaration, Diagnostic> {
        self.advance();
        let name = self.parse_identifier("expected a function name")?;
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after function name"));
        }

        let mut parameters = Vec::new();
        let mut optional_parameter_seen = false;
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightParen) {
            let parameter = self.parse_function_parameter()?;
            if optional_parameter_seen && !parameter.is_optional() {
                self.diagnostics.push(Diagnostic::new(
                    1016,
                    "A required parameter cannot follow an optional parameter.",
                    parameter.span(),
                ));
            }
            optional_parameter_seen |= parameter.is_optional();
            parameters.push(parameter);
            if !self.matches(&TokenKind::Comma) {
                break;
            }
        }
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after function parameters"));
        }

        let return_type = if self.matches(&TokenKind::Colon) {
            Some(self.parse_return_type_reference("expected a function return type")?)
        } else {
            None
        };

        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' before function body"));
        }
        let body = self.parse_function_body()?;

        Ok(FunctionDeclaration {
            name,
            accessibility: None,
            is_static: false,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_function_body(&mut self) -> Result<Vec<FunctionBodyStatement>, Diagnostic> {
        let mut body = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            body.extend(self.parse_function_body_statement()?);
        }
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after function body"));
        }
        Ok(body)
    }

    fn parse_class_declaration(&mut self) -> Result<ClassDeclaration, Diagnostic> {
        self.advance();
        let name = self.parse_identifier("expected a class name")?;
        let (base_class, base_class_span) = if self.matches(&TokenKind::Extends) {
            let (name, span) = self.parse_identifier_with_span("expected a base class name")?;
            (Some(name), Some(span))
        } else {
            (None, None)
        };
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after class name"));
        }
        let mut members = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            if self.matches(&TokenKind::Semicolon) {
                continue;
            }
            members.push(self.parse_class_member()?);
        }
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after class members"));
        }
        Ok(ClassDeclaration {
            name,
            base_class,
            base_class_span,
            members,
        })
    }

    fn parse_class_member(&mut self) -> Result<ClassMember, Diagnostic> {
        let modifiers = self.parse_class_member_modifiers();
        let readonly = modifiers.readonly_span.is_some();
        let (name, name_span) = self.parse_identifier_with_span("expected a class member name")?;
        let is_constructor = name == "constructor";
        if !self.matches(&TokenKind::LeftParen) {
            let type_annotation = if self.matches(&TokenKind::Colon) {
                Some(self.parse_type_reference("expected a class property type")?)
            } else {
                None
            };
            let initializer = if self.matches(&TokenKind::Equals) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.matches(&TokenKind::Semicolon);
            return Ok(ClassMember::Property(PropertyDeclaration {
                name,
                name_span,
                accessibility: modifiers.accessibility,
                is_static: modifiers.is_static,
                readonly,
                type_annotation,
                initializer,
            }));
        }
        let mut parameters = Vec::new();
        let mut optional_parameter_seen = false;
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightParen) {
            let parameter = if is_constructor {
                self.parse_constructor_parameter()?
            } else {
                self.parse_function_parameter()?
            };
            if optional_parameter_seen && !parameter.is_optional() {
                self.diagnostics.push(Diagnostic::new(
                    1016,
                    "A required parameter cannot follow an optional parameter.",
                    parameter.span(),
                ));
            }
            optional_parameter_seen |= parameter.is_optional();
            parameters.push(parameter);
            if !self.matches(&TokenKind::Comma) {
                break;
            }
        }
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after class method parameters"));
        }
        if let Some(span) = modifiers.readonly_span {
            self.diagnostics.push(Diagnostic::new(
                1024,
                "'readonly' modifier can only appear on a property declaration or index signature.",
                span,
            ));
        }
        let return_type = if self.matches(&TokenKind::Colon) {
            Some(self.parse_type_reference("expected a class method return type")?)
        } else {
            None
        };
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' before class method body"));
        }
        let body = self.parse_function_body()?;
        Ok(ClassMember::Method(FunctionDeclaration {
            name,
            accessibility: modifiers.accessibility,
            is_static: modifiers.is_static,
            parameters,
            return_type,
            body,
        }))
    }

    fn parse_function_body_statement(&mut self) -> Result<Vec<FunctionBodyStatement>, Diagnostic> {
        match self.peek().kind {
            TokenKind::Throw => {
                self.advance();
                let expression = self.parse_expression()?;
                self.matches(&TokenKind::Semicolon);
                Ok(vec![FunctionBodyStatement::Throw(expression)])
            }
            TokenKind::Return => {
                let return_span = self.advance().span;
                let expression = if matches!(
                    self.peek().kind,
                    TokenKind::Semicolon | TokenKind::RightBrace
                ) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                self.matches(&TokenKind::Semicolon);
                Ok(vec![FunctionBodyStatement::Return(ReturnStatement {
                    expression,
                    span: return_span,
                })])
            }
            TokenKind::Break => {
                let span = self.advance().span;
                if self.loop_depth == 0 && self.switch_depth == 0 {
                    self.diagnostics.push(Diagnostic::new(
                        1107,
                        "Jump target cannot cross function boundary.",
                        span,
                    ));
                }
                self.matches(&TokenKind::Semicolon);
                Ok(vec![FunctionBodyStatement::Break { span }])
            }
            TokenKind::Continue => {
                let span = self.advance().span;
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::new(
                        1107,
                        "Jump target cannot cross function boundary.",
                        span,
                    ));
                }
                self.matches(&TokenKind::Semicolon);
                Ok(vec![FunctionBodyStatement::Continue { span }])
            }
            TokenKind::Const | TokenKind::Let | TokenKind::Var => {
                let declaration = self.parse_variable_declaration()?;
                let declaration_kind = declaration.declaration_kind();
                let mut declarations =
                    vec![FunctionBodyStatement::VariableDeclaration(declaration)];
                while self.matches(&TokenKind::Comma) {
                    let declaration = self.parse_variable_declarator(declaration_kind)?;
                    declarations.push(FunctionBodyStatement::VariableDeclaration(declaration));
                }
                self.matches(&TokenKind::Semicolon);
                Ok(declarations)
            }
            _ if matches!(&self.peek().kind, TokenKind::Identifier(name) if name == "if") => {
                Ok(vec![self.parse_if_statement()?])
            }
            TokenKind::While => Ok(vec![self.parse_while_statement()?]),
            TokenKind::Do => Ok(vec![self.parse_do_while_statement()?]),
            TokenKind::For => Ok(vec![self.parse_for_statement()?]),
            TokenKind::Switch => Ok(vec![self.parse_switch_statement()?]),
            TokenKind::Try => Ok(vec![self.parse_try_statement()?]),
            _ => {
                let expression = self.parse_expression()?;
                self.matches(&TokenKind::Semicolon);
                Ok(vec![FunctionBodyStatement::Expression(expression)])
            }
        }
    }

    fn is_control_flow_start(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Do
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Switch
                | TokenKind::Try
                | TokenKind::Throw
        ) || matches!(&self.peek().kind, TokenKind::Identifier(name) if name == "if")
    }

    fn parse_top_level_control_flow_statement(&mut self) -> Result<Statement, Diagnostic> {
        let mut statements = self.parse_function_body_statement()?;
        if statements.len() != 1 {
            return Err(self.error("expected one top-level control-flow statement"));
        }
        Ok(Statement::ControlFlowStatement(statements.remove(0)))
    }

    fn parse_if_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        if !self.matches_identifier("if") {
            return Err(self.error("expected 'if'"));
        }
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after 'if'"));
        }
        let condition = self.parse_expression()?;
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after if condition"));
        }
        let then_body = self.parse_function_body_branch()?;
        let else_body = if self.matches_identifier("else") {
            Some(self.parse_function_body_branch()?)
        } else {
            None
        };
        Ok(FunctionBodyStatement::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_while_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        self.advance();
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after 'while'"));
        }
        let condition = self.parse_expression()?;
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after while condition"));
        }
        let body = self.parse_loop_body()?;
        Ok(FunctionBodyStatement::While { condition, body })
    }

    fn parse_do_while_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        self.advance();
        let body = self.parse_loop_body()?;
        if !self.matches(&TokenKind::While) {
            return Err(self.error("expected 'while' after do-while body"));
        }
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after 'while'"));
        }
        let condition = self.parse_expression()?;
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after while condition"));
        }
        self.matches(&TokenKind::Semicolon);
        Ok(FunctionBodyStatement::DoWhile { body, condition })
    }

    fn parse_for_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        self.advance();
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after 'for'"));
        }
        let initializer = if self.matches(&TokenKind::Semicolon) {
            None
        } else {
            let initializer = if matches!(
                self.peek().kind,
                TokenKind::Const | TokenKind::Let | TokenKind::Var
            ) {
                let first = self.parse_variable_declaration()?;
                let declaration_kind = first.declaration_kind();
                let mut declarations = vec![first];
                while self.matches(&TokenKind::Comma) {
                    declarations.push(self.parse_variable_declarator(declaration_kind)?);
                }
                ForInitializer::VariableDeclarations(declarations)
            } else {
                ForInitializer::Expression(self.parse_expression()?)
            };
            if self.matches_identifier("of") {
                if matches!(&initializer, ForInitializer::VariableDeclarations(declarations) if declarations.len() > 1)
                {
                    return Err(self.error("a for-of loop may have only one declaration"));
                }
                let iterable = self.parse_expression()?;
                if !self.matches(&TokenKind::RightParen) {
                    return Err(self.error("expected ')' after for-of expression"));
                }
                let body = self.parse_loop_body()?;
                return Ok(FunctionBodyStatement::ForOf {
                    initializer,
                    iterable,
                    body,
                });
            }
            if self.matches_identifier("in") {
                if matches!(&initializer, ForInitializer::VariableDeclarations(declarations) if declarations.len() > 1)
                {
                    return Err(self.error("a for-in loop may have only one declaration"));
                }
                let object = self.parse_expression()?;
                if !self.matches(&TokenKind::RightParen) {
                    return Err(self.error("expected ')' after for-in expression"));
                }
                let body = self.parse_loop_body()?;
                return Ok(FunctionBodyStatement::ForIn {
                    initializer,
                    object,
                    body,
                });
            }
            if !self.matches(&TokenKind::Semicolon) {
                return Err(self.error("expected ';' after for-loop initializer"));
            }
            Some(initializer)
        };
        let condition = if self.matches(&TokenKind::Semicolon) {
            None
        } else {
            let condition = self.parse_expression()?;
            if !self.matches(&TokenKind::Semicolon) {
                return Err(self.error("expected ';' after for-loop condition"));
            }
            Some(condition)
        };
        let incrementor = if self.matches(&TokenKind::RightParen) {
            None
        } else {
            let incrementor = self.parse_expression()?;
            if !self.matches(&TokenKind::RightParen) {
                return Err(self.error("expected ')' after for-loop clauses"));
            }
            Some(incrementor)
        };
        let body = self.parse_loop_body()?;
        Ok(FunctionBodyStatement::For {
            initializer,
            condition,
            incrementor,
            body,
        })
    }

    fn parse_switch_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        self.advance();
        if !self.matches(&TokenKind::LeftParen) {
            return Err(self.error("expected '(' after 'switch'"));
        }
        let expression = self.parse_expression()?;
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after switch expression"));
        }
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' before switch clauses"));
        }
        self.switch_depth += 1;
        let clauses_result = self.parse_switch_clauses();
        self.switch_depth -= 1;
        let clauses = clauses_result?;
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after switch clauses"));
        }
        Ok(FunctionBodyStatement::Switch {
            expression,
            clauses,
        })
    }

    fn parse_switch_clauses(&mut self) -> Result<Vec<SwitchClause>, Diagnostic> {
        let mut clauses = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let expression = if self.matches_identifier("case") {
                Some(self.parse_expression()?)
            } else if self.matches(&TokenKind::Default) {
                None
            } else {
                return Err(self.error("expected 'case' or 'default' in switch statement"));
            };
            if !self.matches(&TokenKind::Colon) {
                return Err(self.error("expected ':' after switch clause label"));
            }
            let mut statements = Vec::new();
            while !self.at_end()
                && !matches!(self.peek().kind, TokenKind::RightBrace)
                && !self.is_switch_clause_start()
            {
                statements.extend(self.parse_function_body_statement()?);
            }
            clauses.push(SwitchClause {
                expression,
                statements,
            });
        }
        Ok(clauses)
    }

    fn is_switch_clause_start(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Default)
            || matches!(&self.peek().kind, TokenKind::Identifier(name) if name == "case")
    }

    fn parse_try_statement(&mut self) -> Result<FunctionBodyStatement, Diagnostic> {
        self.advance();
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after 'try'"));
        }
        let try_body = self.parse_function_body()?;
        let catch_clause = if self.matches(&TokenKind::Catch) {
            Some(self.parse_catch_clause()?)
        } else {
            None
        };
        let finally_body = if self.matches(&TokenKind::Finally) {
            if !self.matches(&TokenKind::LeftBrace) {
                return Err(self.error("expected '{' after 'finally'"));
            }
            Some(self.parse_function_body()?)
        } else {
            None
        };
        if catch_clause.is_none() && finally_body.is_none() {
            return Err(self.error("expected 'catch' or 'finally' after try block"));
        }
        Ok(FunctionBodyStatement::Try {
            try_body,
            catch_clause,
            finally_body,
        })
    }

    fn parse_catch_clause(&mut self) -> Result<CatchClause, Diagnostic> {
        let variable = if self.matches(&TokenKind::LeftParen) {
            let name_token = self.advance();
            let name_span = name_token.span;
            let TokenKind::Identifier(name) = name_token.kind else {
                return Err(Self::error_at(
                    name_token.span,
                    "expected a catch binding name",
                ));
            };
            let type_annotation = if self.matches(&TokenKind::Colon) {
                Some(self.parse_type_reference("expected a catch binding type")?)
            } else {
                None
            };
            if !self.matches(&TokenKind::RightParen) {
                return Err(self.error("expected ')' after catch binding"));
            }
            Some(VariableDeclaration {
                declaration_kind: VariableDeclarationKind::Let,
                name,
                name_span,
                type_annotation,
                initializer: None,
            })
        } else {
            None
        };
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after catch clause"));
        }
        let body = self.parse_function_body()?;
        Ok(CatchClause { variable, body })
    }

    fn parse_loop_body(&mut self) -> Result<Vec<FunctionBodyStatement>, Diagnostic> {
        self.loop_depth += 1;
        let body_result = self.parse_function_body_branch();
        self.loop_depth -= 1;
        body_result
    }

    fn parse_function_body_branch(&mut self) -> Result<Vec<FunctionBodyStatement>, Diagnostic> {
        if self.matches(&TokenKind::LeftBrace) {
            self.parse_function_body()
        } else {
            self.parse_function_body_statement()
        }
    }

    fn parse_import_declaration(&mut self) -> Result<ImportDeclaration, Diagnostic> {
        self.advance();
        let type_only = self.matches(&TokenKind::Type);
        let mut default_import = None;
        let mut namespace_import = None;
        let mut named_imports = Vec::new();
        if self.matches(&TokenKind::LeftBrace) {
            named_imports = self.parse_named_import_specifiers()?;
        } else if self.matches(&TokenKind::Asterisk) {
            namespace_import = Some(self.parse_namespace_import_specifier()?);
        } else if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let (local_name, span) =
                self.parse_identifier_with_span("expected a default import name")?;
            default_import = Some(ImportSpecifier {
                imported_name: "default".to_owned(),
                local_name,
                span,
            });
            if self.matches(&TokenKind::Comma) {
                if self.matches(&TokenKind::LeftBrace) {
                    named_imports = self.parse_named_import_specifiers()?;
                } else if self.matches(&TokenKind::Asterisk) {
                    namespace_import = Some(self.parse_namespace_import_specifier()?);
                } else {
                    return Err(self.error("expected named or namespace imports after ','"));
                }
            }
        }
        if (default_import.is_some() || namespace_import.is_some() || !named_imports.is_empty())
            && !self.matches_identifier("from")
        {
            return Err(self.error("expected 'from' after import bindings"));
        }
        let (module_specifier, raw_module_specifier, span) =
            Self::parse_quoted_module_specifier(self.advance())?;
        self.matches(&TokenKind::Semicolon);
        Ok(ImportDeclaration {
            module_specifier,
            raw_module_specifier,
            type_only,
            default_import,
            namespace_import,
            named_imports,
            span,
        })
    }

    fn parse_namespace_import_specifier(&mut self) -> Result<ImportSpecifier, Diagnostic> {
        if !self.matches_identifier("as") {
            return Err(self.error("expected 'as' after namespace import '*'"));
        }
        let (local_name, span) =
            self.parse_identifier_with_span("expected a namespace import name after 'as'")?;
        Ok(ImportSpecifier {
            imported_name: "*".to_owned(),
            local_name,
            span,
        })
    }

    fn parse_named_import_specifiers(&mut self) -> Result<Vec<ImportSpecifier>, Diagnostic> {
        let mut specifiers = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let (imported_name, span) =
                self.parse_identifier_with_span("expected an imported name")?;
            let local_name = if self.matches_identifier("as") {
                self.parse_identifier("expected a local import name after 'as'")?
            } else {
                imported_name.clone()
            };
            specifiers.push(ImportSpecifier {
                imported_name,
                local_name,
                span,
            });
            if !self.matches(&TokenKind::Comma)
                && !matches!(self.peek().kind, TokenKind::RightBrace)
            {
                return Err(self.error("expected ',' between named imports"));
            }
        }
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after named imports"));
        }
        Ok(specifiers)
    }

    fn parse_quoted_module_specifier(
        module_token: Token,
    ) -> Result<(String, String, TextSpan), Diagnostic> {
        let Token { kind, span } = module_token;
        let TokenKind::StringLiteral(raw_module_specifier) = kind else {
            return Err(Self::error_at(
                span,
                "expected a string module specifier after import",
            ));
        };
        if raw_module_specifier.starts_with('`') {
            return Err(Self::error_at(
                span,
                "expected a quoted string module specifier after import",
            ));
        }
        let Some(quote) = raw_module_specifier.chars().next() else {
            return Err(Self::error_at(
                span,
                "expected a quoted string module specifier after import",
            ));
        };
        if !raw_module_specifier.ends_with(quote) {
            return Err(Self::error_at(
                span,
                "expected a closing quote on the module specifier",
            ));
        }
        let quote_width = quote.len_utf8();
        if raw_module_specifier.len() < quote_width * 2 {
            return Err(Self::error_at(
                span,
                "expected a closing quote on the module specifier",
            ));
        }
        let content_end = raw_module_specifier.len() - quote_width;
        let module_specifier = raw_module_specifier[quote_width..content_end].to_owned();
        Ok((module_specifier, raw_module_specifier, span))
    }

    fn parse_named_export_declaration(&mut self, type_only: bool) -> Result<Statement, Diagnostic> {
        let mut specifiers = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let (local_name, span) =
                self.parse_identifier_with_span("expected a local name in export list")?;
            let exported_name = if self.matches_identifier("as") {
                self.parse_identifier("expected an exported name after 'as'")?
            } else {
                local_name.clone()
            };
            specifiers.push(ExportSpecifier {
                local_name,
                exported_name,
                span,
            });
            if !self.matches(&TokenKind::Comma)
                && !matches!(self.peek().kind, TokenKind::RightBrace)
            {
                return Err(self.error("expected ',' between exported names"));
            }
        }
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after named exports"));
        }
        if self.matches_identifier("from") {
            let (module_specifier, raw_module_specifier, span) =
                Self::parse_quoted_module_specifier(self.advance())?;
            self.matches(&TokenKind::Semicolon);
            return Ok(Statement::ExportNamedFrom(ExportNamedFromDeclaration {
                specifiers,
                module_specifier,
                raw_module_specifier,
                span,
                type_only,
            }));
        }
        self.matches(&TokenKind::Semicolon);
        Ok(if type_only {
            Statement::ExportTypeNamed(specifiers)
        } else {
            Statement::ExportNamed(specifiers)
        })
    }

    fn parse_export_all_declaration(&mut self) -> Result<Statement, Diagnostic> {
        if !self.matches_identifier("from") {
            return Err(self.error("expected 'from' after export '*'"));
        }
        let (module_specifier, raw_module_specifier, span) =
            Self::parse_quoted_module_specifier(self.advance())?;
        self.matches(&TokenKind::Semicolon);
        Ok(Statement::ExportAll(ExportAllDeclaration {
            module_specifier,
            raw_module_specifier,
            span,
        }))
    }

    fn parse_function_parameter(&mut self) -> Result<FunctionParameter, Diagnostic> {
        self.parse_function_parameter_with_property(None, false)
    }

    fn parse_constructor_parameter(&mut self) -> Result<FunctionParameter, Diagnostic> {
        let modifiers = self.parse_parameter_property_modifiers();
        self.parse_function_parameter_with_property(
            modifiers.accessibility,
            modifiers.readonly_span.is_some(),
        )
    }

    fn parse_function_parameter_with_property(
        &mut self,
        parameter_property_accessibility: Option<super::ast::MemberAccessibility>,
        parameter_property_readonly: bool,
    ) -> Result<FunctionParameter, Diagnostic> {
        let (name, span) = self.parse_identifier_with_span("expected a parameter name")?;
        let optional = self.matches(&TokenKind::Question);
        let type_annotation = if self.matches(&TokenKind::Colon) {
            Some(self.parse_type_reference("expected a parameter type")?)
        } else {
            None
        };
        let initializer = if self.matches(&TokenKind::Equals) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(FunctionParameter {
            name,
            optional,
            type_annotation,
            initializer,
            parameter_property_accessibility,
            parameter_property_readonly,
            span,
        })
    }

    fn parse_interface_declaration(&mut self) -> Result<InterfaceDeclaration, Diagnostic> {
        self.advance();
        let name = self.parse_identifier("expected an interface name")?;
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after interface name"));
        }

        let mut members = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            members.push(self.parse_property_signature()?);
            self.matches(&TokenKind::Semicolon);
            self.matches(&TokenKind::Comma);
        }

        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after interface members"));
        }

        Ok(InterfaceDeclaration { name, members })
    }

    fn parse_namespace_declaration(&mut self) -> Result<NamespaceDeclaration, Diagnostic> {
        let start = self.advance().span.start().get();
        let mut names = vec![self.parse_identifier_with_span("expected a namespace name")?];
        while self.matches(&TokenKind::Dot) {
            names.push(self.parse_identifier_with_span("expected a namespace name after '.'")?);
        }
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after namespace name"));
        }

        let mut members = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let statement = self.parse_statement()?;
            let declaration_kind = variable_declaration_kind(&statement);
            members.push(statement);
            if let Some((declaration_kind, exported)) = declaration_kind {
                while self.matches(&TokenKind::Comma) {
                    let declaration = self.parse_variable_declarator(declaration_kind)?;
                    let statement = Statement::VariableDeclaration(declaration);
                    members.push(if exported {
                        Statement::ExportedDeclaration(Box::new(statement))
                    } else {
                        statement
                    });
                }
                self.matches(&TokenKind::Semicolon);
            }
        }

        let closing_span = self.peek().span;
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after namespace members"));
        }
        self.matches(&TokenKind::Semicolon);
        let end = closing_span.start().get() + closing_span.length();
        let (name, name_span) = names
            .pop()
            .expect("a namespace declaration has at least one name segment");
        let mut declaration = NamespaceDeclaration {
            name,
            name_span,
            members,
            span: TextSpan::new(
                Utf16Offset::new(name_span.start().get()),
                end - name_span.start().get(),
            ),
        };
        while let Some((name, name_span)) = names.pop() {
            let nested = Statement::ExportedDeclaration(Box::new(Statement::NamespaceDeclaration(
                declaration,
            )));
            declaration = NamespaceDeclaration {
                name,
                name_span,
                members: vec![nested],
                span: TextSpan::new(
                    Utf16Offset::new(name_span.start().get()),
                    end - name_span.start().get(),
                ),
            };
        }
        declaration.span = TextSpan::new(Utf16Offset::new(start), end - start);
        Ok(declaration)
    }

    fn parse_enum_declaration(&mut self, is_const: bool) -> Result<EnumDeclaration, Diagnostic> {
        self.advance();
        let name = self.parse_identifier("expected an enum name")?;
        if !self.matches(&TokenKind::LeftBrace) {
            return Err(self.error("expected '{' after enum name"));
        }

        let mut members = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let (name, name_span) =
                self.parse_identifier_with_span("expected an enum member name")?;
            let initializer = if self.matches(&TokenKind::Equals) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            members.push(EnumMember {
                name,
                name_span,
                initializer,
            });
            if !self.matches(&TokenKind::Comma) && !self.matches(&TokenKind::Semicolon) {
                break;
            }
        }

        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after enum members"));
        }
        self.matches(&TokenKind::Semicolon);
        Ok(EnumDeclaration {
            name,
            members,
            is_const,
        })
    }

    fn parse_type_alias_declaration(&mut self) -> Result<TypeAliasDeclaration, Diagnostic> {
        self.advance();
        let name = self.parse_identifier("expected a type alias name")?;
        if !self.matches(&TokenKind::Equals) {
            return Err(self.error("expected '=' after type alias name"));
        }

        let type_annotation = self.parse_type_reference("expected an aliased type name")?;
        self.matches(&TokenKind::Semicolon);

        Ok(TypeAliasDeclaration {
            name,
            type_annotation,
        })
    }

    fn parse_property_signature(&mut self) -> Result<PropertySignature, Diagnostic> {
        let name = self.parse_identifier("expected an interface property name")?;
        let optional = self.matches(&TokenKind::Question);
        if !self.matches(&TokenKind::Colon) {
            return Err(self.error("expected ':' after interface property name"));
        }
        let type_annotation = self.parse_type_reference("expected a property type name")?;

        Ok(PropertySignature {
            name,
            optional,
            type_annotation,
        })
    }

    fn parse_identifier(&mut self, message: &str) -> Result<String, Diagnostic> {
        self.parse_identifier_with_span(message)
            .map(|(name, _)| name)
    }

    fn parse_identifier_with_span(
        &mut self,
        message: &str,
    ) -> Result<(String, TextSpan), Diagnostic> {
        let token = self.advance();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Self::error_at(token.span, message));
        };
        Ok((name, token.span))
    }

    fn parse_type_reference(&mut self, message: &str) -> Result<TypeReference, Diagnostic> {
        let start = self.peek().span.start();
        let (first_name, first_array_dimensions, mut end) = self.parse_type_member(message)?;
        let mut names = vec![first_name];
        let mut array_dimensions = vec![first_array_dimensions];

        while self.matches(&TokenKind::Pipe) {
            let (name, dimensions, member_end) = self.parse_type_member(message)?;
            end = member_end;
            names.push(name);
            array_dimensions.push(dimensions);
        }

        Ok(TypeReference {
            names,
            array_dimensions,
            predicate_parameter: None,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn parse_return_type_reference(&mut self, message: &str) -> Result<TypeReference, Diagnostic> {
        let mut return_type = self.parse_type_reference(message)?;
        if !matches!(&self.peek().kind, TokenKind::Identifier(name) if name == "is")
            || return_type.names.len() != 1
            || return_type.array_dimensions.first() != Some(&0)
        {
            return Ok(return_type);
        }

        let parameter_name = return_type.names[0].clone();
        self.advance();
        let predicate_type = self.parse_type_reference(message)?;
        let start = return_type.span.start();
        let end = predicate_type.span.start().get() + predicate_type.span.length();
        return_type.names = predicate_type.names;
        return_type.array_dimensions = predicate_type.array_dimensions;
        return_type.predicate_parameter = Some(parameter_name);
        return_type.span = TextSpan::new(start, end - start.get());
        Ok(return_type)
    }

    fn parse_type_member(&mut self, message: &str) -> Result<(String, usize, usize), Diagnostic> {
        let token = self.advance();
        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            TokenKind::NullLiteral => "null".to_owned(),
            _ => return Err(Self::error_at(token.span, message)),
        };
        let mut end = token.span.start().get() + token.span.length();
        let mut dimensions = 0;
        while self.matches(&TokenKind::LeftBracket) {
            let closing_span = self.peek().span;
            if !self.matches(&TokenKind::RightBracket) {
                return Err(self.error("expected ']' after array type"));
            }
            dimensions += 1;
            end = closing_span.start().get() + closing_span.length();
        }
        Ok((name, dimensions, end))
    }

    fn parse_variable_declaration(&mut self) -> Result<VariableDeclaration, Diagnostic> {
        let declaration_kind = match self.advance().kind {
            TokenKind::Const => VariableDeclarationKind::Const,
            TokenKind::Let => VariableDeclarationKind::Let,
            TokenKind::Var => VariableDeclarationKind::Var,
            _ => unreachable!("the parser checks the declaration keyword before consuming it"),
        };

        self.parse_variable_declarator(declaration_kind)
    }

    fn parse_variable_declarator(
        &mut self,
        declaration_kind: VariableDeclarationKind,
    ) -> Result<VariableDeclaration, Diagnostic> {
        let name_token = self.advance();
        let name_span = name_token.span;
        let TokenKind::Identifier(name) = name_token.kind else {
            return Err(Self::error_at(name_token.span, "expected a binding name"));
        };

        let type_annotation = if self.matches(&TokenKind::Colon) {
            Some(self.parse_type_reference("expected a type name")?)
        } else {
            None
        };

        let initializer = if self.matches(&TokenKind::Equals) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(VariableDeclaration {
            declaration_kind,
            name,
            name_span,
            type_annotation,
            initializer,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, Diagnostic> {
        let left = self.parse_binary_expression()?;
        if matches!(self.peek().kind, TokenKind::Question) {
            self.advance();
            let when_true = self.parse_expression()?;
            if !self.matches(&TokenKind::Colon) {
                return Err(self.error("expected ':' in conditional expression"));
            }
            let when_false = self.parse_expression()?;
            let start = left.span().start();
            let end = when_false.span().start().get() + when_false.span().length();
            return Ok(Expression::ConditionalExpression {
                condition: Box::new(left),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
                span: TextSpan::new(start, end - start.get()),
            });
        }

        let operator = match &self.peek().kind {
            TokenKind::Equals => AssignmentOperator::Assign,
            TokenKind::PlusEquals => AssignmentOperator::AddAssign,
            _ => return Ok(left),
        };
        let operator_span = self.advance().span;

        let right = self.parse_expression()?;
        let start = left.span().start();
        let end = right.span().start().get() + right.span().length();
        Ok(Expression::AssignmentExpression {
            left: Box::new(left),
            right: Box::new(right),
            operator,
            operator_span,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn parse_binary_expression(&mut self) -> Result<Expression, Diagnostic> {
        self.parse_nullish_coalescing_expression()
    }

    fn parse_nullish_coalescing_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_logical_or_expression()?;
        while matches!(self.peek().kind, TokenKind::QuestionQuestion) {
            let operator_token = self.advance();
            let right = self.parse_logical_or_expression()?;
            if let Some(logical_operator) = Self::unparenthesized_logical_operator(&expression)
                .or_else(|| Self::unparenthesized_logical_operator(&right))
            {
                self.diagnostics.push(Diagnostic::new(
                    5076,
                    format!(
                        "'??' and '{}' operations cannot be mixed without parentheses.",
                        logical_operator.as_str()
                    ),
                    operator_token.span,
                ));
            }
            expression = Self::binary_expression(
                expression,
                BinaryOperator::NullishCoalesce,
                operator_token.span,
                right,
            );
        }
        Ok(expression)
    }

    fn unparenthesized_logical_operator(expression: &Expression) -> Option<BinaryOperator> {
        match expression {
            Expression::BinaryExpression { operator, .. }
                if matches!(
                    operator,
                    BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
                ) =>
            {
                Some(*operator)
            }
            Expression::NumberLiteral { .. }
            | Expression::BooleanLiteral { .. }
            | Expression::NullLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::BinaryExpression { .. }
            | Expression::ConditionalExpression { .. }
            | Expression::AssignmentExpression { .. }
            | Expression::ParenthesizedExpression { .. }
            | Expression::TypeAssertionExpression { .. }
            | Expression::CallExpression { .. }
            | Expression::NewExpression { .. }
            | Expression::ArrowFunction { .. }
            | Expression::PropertyAccessExpression { .. }
            | Expression::ElementAccessExpression { .. }
            | Expression::UnaryExpression { .. }
            | Expression::ObjectLiteral { .. }
            | Expression::ArrayLiteral { .. } => None,
        }
    }

    fn parse_logical_or_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_logical_and_expression()?;
        while matches!(self.peek().kind, TokenKind::PipePipe) {
            let operator_token = self.advance();
            let right = self.parse_logical_and_expression()?;
            expression = Self::binary_expression(
                expression,
                BinaryOperator::LogicalOr,
                operator_token.span,
                right,
            );
        }
        Ok(expression)
    }

    fn parse_logical_and_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_equality_expression()?;
        while matches!(self.peek().kind, TokenKind::AmpersandAmpersand) {
            let operator_token = self.advance();
            let right = self.parse_equality_expression()?;
            expression = Self::binary_expression(
                expression,
                BinaryOperator::LogicalAnd,
                operator_token.span,
                right,
            );
        }
        Ok(expression)
    }

    fn parse_equality_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_relational_expression()?;
        while matches!(
            self.peek().kind,
            TokenKind::EqualsEquals
                | TokenKind::EqualsEqualsEquals
                | TokenKind::BangEquals
                | TokenKind::BangEqualsEquals
        ) {
            let operator_token = self.advance();
            let operator = match operator_token.kind {
                TokenKind::EqualsEquals => BinaryOperator::Equal,
                TokenKind::EqualsEqualsEquals => BinaryOperator::StrictEqual,
                TokenKind::BangEquals => BinaryOperator::NotEqual,
                TokenKind::BangEqualsEquals => BinaryOperator::StrictNotEqual,
                _ => unreachable!("the parser checks the equality operator before consuming it"),
            };
            let right = self.parse_relational_expression()?;
            expression = Self::binary_expression(expression, operator, operator_token.span, right);
        }
        Ok(expression)
    }

    fn parse_relational_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_additive_expression()?;
        while matches!(
            self.peek().kind,
            TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::LessThanEquals
                | TokenKind::GreaterThanEquals
        ) {
            let operator_token = self.advance();
            let operator = match operator_token.kind {
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::LessThanEquals => BinaryOperator::LessThanOrEqual,
                TokenKind::GreaterThanEquals => BinaryOperator::GreaterThanOrEqual,
                _ => unreachable!("the parser checks the relational operator before consuming it"),
            };
            let right = self.parse_additive_expression()?;
            expression = Self::binary_expression(expression, operator, operator_token.span, right);
        }
        Ok(expression)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_multiplicative_expression()?;
        while matches!(self.peek().kind, TokenKind::Plus | TokenKind::Minus) {
            let operator_token = self.advance();
            let operator = match operator_token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => unreachable!("the parser checks the arithmetic operator before consuming it"),
            };
            let right = self.parse_multiplicative_expression()?;
            expression = Self::binary_expression(expression, operator, operator_token.span, right);
        }
        Ok(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_unary_expression()?;
        while matches!(
            self.peek().kind,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent
        ) {
            let operator_token = self.advance();
            let operator = match operator_token.kind {
                TokenKind::Asterisk => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Remainder,
                _ => unreachable!(
                    "the parser checks the multiplicative operator before consuming it"
                ),
            };
            let right = self.parse_unary_expression()?;
            expression = Self::binary_expression(expression, operator, operator_token.span, right);
        }
        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, Diagnostic> {
        if matches!(
            self.peek().kind,
            TokenKind::Plus | TokenKind::Minus | TokenKind::Bang
        ) {
            let operator_token = self.advance();
            let operator = match operator_token.kind {
                TokenKind::Plus => UnaryOperator::Plus,
                TokenKind::Minus => UnaryOperator::Negate,
                TokenKind::Bang => UnaryOperator::LogicalNot,
                _ => unreachable!("the parser checks the unary operator before consuming it"),
            };
            let operand = self.parse_unary_expression()?;
            let start = operator_token.span.start();
            let end = operand.span().start().get() + operand.span().length();
            return Ok(Expression::UnaryExpression {
                operator,
                operand: Box::new(operand),
                span: TextSpan::new(start, end - start.get()),
            });
        }

        self.parse_postfix_expression()
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_primary_expression()?;
        loop {
            if self.matches(&TokenKind::LeftParen) {
                let mut arguments = Vec::new();
                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.matches(&TokenKind::Comma)
                            || matches!(self.peek().kind, TokenKind::RightParen)
                        {
                            break;
                        }
                    }
                }
                let closing_span = self.peek().span;
                if !self.matches(&TokenKind::RightParen) {
                    return Err(self.error("expected ')' after call arguments"));
                }
                let start = expression.span().start();
                let end = closing_span.start().get() + closing_span.length();
                expression = Expression::CallExpression {
                    callee: Box::new(expression),
                    arguments,
                    span: TextSpan::new(start, end - start.get()),
                };
                continue;
            }

            if self.matches(&TokenKind::LeftBracket) {
                let argument = self.parse_expression()?;
                let closing_span = self.peek().span;
                if !self.matches(&TokenKind::RightBracket) {
                    return Err(self.error("expected ']' after element access"));
                }
                let start = expression.span().start();
                let end = closing_span.start().get() + closing_span.length();
                expression = Expression::ElementAccessExpression {
                    receiver: Box::new(expression),
                    argument: Box::new(argument),
                    span: TextSpan::new(start, end - start.get()),
                };
                continue;
            }

            if self.matches(&TokenKind::Dot) {
                let (name, name_span) =
                    self.parse_identifier_with_span("expected a property name after '.'")?;
                let start = expression.span().start();
                let end = name_span.start().get() + name_span.length();
                expression = Expression::PropertyAccessExpression {
                    receiver: Box::new(expression),
                    name,
                    name_span,
                    span: TextSpan::new(start, end - start.get()),
                };
                continue;
            }

            if matches!(&self.peek().kind, TokenKind::Identifier(name) if name == "as") {
                self.advance();
                let type_annotation =
                    self.parse_type_reference("expected a type name after 'as'")?;
                let start = expression.span().start();
                let end = type_annotation.span().start().get() + type_annotation.span().length();
                expression = Expression::TypeAssertionExpression {
                    expression: Box::new(expression),
                    type_annotation,
                    span: TextSpan::new(start, end - start.get()),
                };
                continue;
            }

            break;
        }
        Ok(expression)
    }

    fn binary_expression(
        left: Expression,
        operator: BinaryOperator,
        operator_span: TextSpan,
        right: Expression,
    ) -> Expression {
        let start = left.span().start();
        let end = right.span().start().get() + right.span().length();
        Expression::BinaryExpression {
            left: Box::new(left),
            operator,
            operator_span,
            right: Box::new(right),
            span: TextSpan::new(start, end - start.get()),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::NumberLiteral(value) => Ok(Expression::NumberLiteral {
                value,
                span: token.span,
            }),
            TokenKind::BooleanLiteral(value) => Ok(Expression::BooleanLiteral {
                value,
                span: token.span,
            }),
            TokenKind::NullLiteral => Ok(Expression::NullLiteral { span: token.span }),
            TokenKind::StringLiteral(raw) => Ok(Expression::StringLiteral {
                raw,
                span: token.span,
            }),
            TokenKind::New => self.parse_new_expression(token.span.start()),
            TokenKind::Identifier(name) if self.is_single_parameter_arrow_start() => {
                self.parse_single_parameter_arrow(name, token.span)
            }
            TokenKind::Identifier(name) => Ok(Expression::Identifier {
                name,
                span: token.span,
            }),
            TokenKind::LeftParen if self.is_parenthesized_arrow_start() => {
                self.parse_arrow_function(token.span.start())
            }
            TokenKind::LeftParen => self.parse_parenthesized_expression(token.span.start()),
            TokenKind::LeftBrace => self.parse_object_literal(token.span.start()),
            TokenKind::LeftBracket => self.parse_array_literal(token.span.start()),
            _ => Err(Self::error_at(token.span, "expected an expression")),
        }
    }

    fn parse_new_expression(&mut self, start: Utf16Offset) -> Result<Expression, Diagnostic> {
        let mut constructor = self.parse_primary_expression()?;
        while self.matches(&TokenKind::Dot) {
            let (name, name_span) =
                self.parse_identifier_with_span("expected a constructor property name after '.'")?;
            let constructor_start = constructor.span().start();
            let end = name_span.start().get() + name_span.length();
            constructor = Expression::PropertyAccessExpression {
                receiver: Box::new(constructor),
                name,
                name_span,
                span: TextSpan::new(constructor_start, end - constructor_start.get()),
            };
        }

        let (arguments, end) = if self.matches(&TokenKind::LeftParen) {
            let mut arguments = Vec::new();
            if !matches!(self.peek().kind, TokenKind::RightParen) {
                loop {
                    arguments.push(self.parse_expression()?);
                    if !self.matches(&TokenKind::Comma)
                        || matches!(self.peek().kind, TokenKind::RightParen)
                    {
                        break;
                    }
                }
            }
            let closing_span = self.peek().span;
            if !self.matches(&TokenKind::RightParen) {
                return Err(self.error("expected ')' after constructor arguments"));
            }
            (
                arguments,
                closing_span.start().get() + closing_span.length(),
            )
        } else {
            (
                Vec::new(),
                constructor.span().start().get() + constructor.span().length(),
            )
        };
        Ok(Expression::NewExpression {
            constructor: Box::new(constructor),
            arguments,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn parse_parenthesized_expression(
        &mut self,
        start: Utf16Offset,
    ) -> Result<Expression, Diagnostic> {
        let expression = self.parse_expression()?;
        let closing_span = self.peek().span;
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after expression"));
        }
        let end = closing_span.start().get() + closing_span.length();
        Ok(Expression::ParenthesizedExpression {
            expression: Box::new(expression),
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn is_parenthesized_arrow_start(&self) -> bool {
        let mut depth = 1_usize;
        let mut index = self.current;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        index += 1;
                        break;
                    }
                }
                TokenKind::EndOfFile => return false,
                _ => {}
            }
            index += 1;
        }
        if depth != 0 {
            return false;
        }

        match self.tokens.get(index).map(|token| &token.kind) {
            Some(TokenKind::Arrow) => true,
            Some(TokenKind::Colon) => self.tokens[index + 1..]
                .iter()
                .take_while(|token| !matches!(&token.kind, TokenKind::EndOfFile))
                .any(|token| matches!(&token.kind, TokenKind::Arrow)),
            _ => false,
        }
    }

    fn parse_arrow_function(&mut self, start: Utf16Offset) -> Result<Expression, Diagnostic> {
        let mut parameters = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightParen) {
            parameters.push(self.parse_function_parameter()?);
            if !self.matches(&TokenKind::Comma) {
                break;
            }
        }
        if !self.matches(&TokenKind::RightParen) {
            return Err(self.error("expected ')' after arrow function parameters"));
        }
        let return_type = if self.matches(&TokenKind::Colon) {
            Some(self.parse_type_reference("expected an arrow function return type")?)
        } else {
            None
        };
        if !self.matches(&TokenKind::Arrow) {
            return Err(self.error("expected '=>' after arrow function parameters"));
        }
        let (body, end) = self.parse_arrow_function_body()?;
        Ok(Expression::ArrowFunction {
            parameters,
            parameters_parenthesized: true,
            return_type,
            body,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn is_single_parameter_arrow_start(&self) -> bool {
        matches!(&self.peek().kind, TokenKind::Arrow)
    }

    fn parse_single_parameter_arrow(
        &mut self,
        name: String,
        parameter_span: TextSpan,
    ) -> Result<Expression, Diagnostic> {
        if !self.matches(&TokenKind::Arrow) {
            return Err(self.error("expected '=>' after arrow function parameter"));
        }
        let start = parameter_span.start();
        let (body, end) = self.parse_arrow_function_body()?;
        Ok(Expression::ArrowFunction {
            parameters: vec![FunctionParameter {
                name,
                optional: false,
                type_annotation: None,
                initializer: None,
                parameter_property_accessibility: None,
                parameter_property_readonly: false,
                span: parameter_span,
            }],
            parameters_parenthesized: false,
            return_type: None,
            body,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn parse_arrow_function_body(&mut self) -> Result<(ArrowFunctionBody, usize), Diagnostic> {
        if self.matches(&TokenKind::LeftBrace) {
            let statements = self.parse_function_body()?;
            let closing_span = self.tokens[self.current - 1].span;
            Ok((
                ArrowFunctionBody::Block(statements),
                closing_span.start().get() + closing_span.length(),
            ))
        } else {
            let expression = Box::new(self.parse_expression()?);
            let end = expression.span().start().get() + expression.span().length();
            Ok((ArrowFunctionBody::Expression(expression), end))
        }
    }

    fn parse_object_literal(&mut self, start: Utf16Offset) -> Result<Expression, Diagnostic> {
        let mut properties = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBrace) {
            let (name, name_span) =
                self.parse_identifier_with_span("expected an object property name")?;
            if !self.matches(&TokenKind::Colon) {
                return Err(self.error("expected ':' after object property name"));
            }
            let value = self.parse_expression()?;
            properties.push(ObjectProperty {
                name,
                name_span,
                value,
            });
            if !self.matches(&TokenKind::Comma)
                && !matches!(self.peek().kind, TokenKind::RightBrace)
            {
                return Err(self.error("expected ',' between object properties"));
            }
        }

        let closing_span = self.peek().span;
        if !self.matches(&TokenKind::RightBrace) {
            return Err(self.error("expected '}' after object properties"));
        }
        let end = closing_span.start().get() + closing_span.length();
        Ok(Expression::ObjectLiteral {
            properties,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn parse_array_literal(&mut self, start: Utf16Offset) -> Result<Expression, Diagnostic> {
        let mut elements = Vec::new();
        while !self.at_end() && !matches!(self.peek().kind, TokenKind::RightBracket) {
            elements.push(self.parse_expression()?);
            if !self.matches(&TokenKind::Comma)
                && !matches!(self.peek().kind, TokenKind::RightBracket)
            {
                return Err(self.error("expected ',' between array elements"));
            }
        }

        let closing_span = self.peek().span;
        if !self.matches(&TokenKind::RightBracket) {
            return Err(self.error("expected ']' after array elements"));
        }
        let end = closing_span.start().get() + closing_span.length();
        Ok(Expression::ArrayLiteral {
            elements,
            span: TextSpan::new(start, end - start.get()),
        })
    }

    fn matches(&mut self, expected: &TokenKind) -> bool {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches_identifier(&mut self, expected: &str) -> bool {
        if matches!(&self.peek().kind, TokenKind::Identifier(name) if name == expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&self, message: &str) -> Diagnostic {
        Self::error_at(self.peek().span, message)
    }

    fn error_at(span: TextSpan, message: &str) -> Diagnostic {
        Diagnostic::new(1005, message, span)
    }

    fn synchronize(&mut self) {
        while !self.at_end() {
            if self.matches(&TokenKind::Semicolon) {
                return;
            }
            if matches!(
                self.peek().kind,
                TokenKind::Const
                    | TokenKind::Let
                    | TokenKind::Var
                    | TokenKind::Interface
                    | TokenKind::Type
                    | TokenKind::Function
                    | TokenKind::Export
            ) {
                return;
            }
            self.advance();
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EndOfFile)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();
        if !self.at_end() {
            self.current += 1;
        }
        token
    }
}

fn variable_declaration_kind(statement: &Statement) -> Option<(VariableDeclarationKind, bool)> {
    statement
        .declaration()
        .as_variable_declaration()
        .map(|declaration| (declaration.declaration_kind(), statement.is_exported()))
}
