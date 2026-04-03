//! AST resolver — walks the full_moon AST to build scopes and symbols.
//!
//! This is the core semantic analysis pass. It visits the AST, tracks scopes,
//! registers symbols, and resolves references.

use std::collections::HashSet;
use std::path::Path;

use full_moon::ast::{self, Ast};

use luagh_core::{Diagnostic, LineIndex, LuaVersion, Position, Span, SymbolKind};

use crate::scope::{ScopeId, ScopeKind, ScopeTree};
use crate::symbol::{Symbol, SymbolId, SymbolTable};

/// The resolver walks the AST and builds the scope tree + symbol table.
#[allow(dead_code)]
pub struct Resolver<'a> {
    lua_version: LuaVersion,
    line_index: &'a LineIndex,
    scopes: ScopeTree,
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,

    /// Stack of active scope IDs (current scope is last).
    scope_stack: Vec<ScopeId>,

    /// Known standard library globals for this Lua version.
    std_globals: HashSet<&'static str>,

    /// File path for diagnostics.
    file_path: std::path::PathBuf,
}

impl<'a> Resolver<'a> {
    pub fn new(lua_version: LuaVersion, line_index: &'a LineIndex) -> Self {
        let std_globals: HashSet<&'static str> =
            luagh_core::std_globals(lua_version).into_iter().collect();

        Self {
            lua_version,
            line_index,
            scopes: ScopeTree::new(),
            symbols: SymbolTable::new(),
            diagnostics: Vec::new(),
            scope_stack: Vec::new(),
            std_globals,
            file_path: std::path::PathBuf::new(),
        }
    }

    /// Run the analysis on the AST.
    pub fn analyze(&mut self, ast: &Ast, file_path: &Path) {
        self.file_path = file_path.to_path_buf();

        // Create the module (file-level) scope
        let module_scope = self.scopes.push(None, ScopeKind::Module);
        self.scope_stack.push(module_scope);

        // Walk the AST
        // NOTE: full_moon's Visitor trait takes &self, so we use an inner
        // pattern if needed. For the scaffold, we do a basic statement walk.
        self.visit_block(ast.nodes());

        // Pop module scope and finalize
        self.scope_stack.pop();
    }

    fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().expect("scope stack is empty")
    }

    fn push_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let parent = Some(self.current_scope());
        let id = self.scopes.push(parent, kind);
        self.scope_stack.push(id);
        id
    }

    fn pop_scope(&mut self) -> ScopeId {
        self.scope_stack.pop().expect("scope stack underflow")
    }

    fn define_symbol(&mut self, name: String, kind: SymbolKind, span: Span) -> SymbolId {
        let scope_id = self.current_scope();
        let symbol = Symbol {
            id: 0, // Will be set by SymbolTable::add
            name: name.clone(),
            kind,
            def_span: span,
            scope_id,
            uses: Vec::new(),
            is_parameter: matches!(kind, SymbolKind::Parameter),
            is_implicit: false,
        };
        let id = self.symbols.add(symbol);
        if let Some(scope) = self.scopes.get_mut(scope_id) {
            scope.symbols.insert(name, id);
        }
        id
    }

    fn visit_block(&mut self, block: &ast::Block) {
        for stmt in block.stmts() {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::LocalAssignment(local_assign) => {
                self.visit_local_assignment(local_assign);
            }
            ast::Stmt::FunctionDeclaration(func_decl) => {
                self.visit_function_declaration(func_decl);
            }
            ast::Stmt::LocalFunction(local_func) => {
                self.visit_local_function(local_func);
            }
            ast::Stmt::Assignment(assign) => {
                self.visit_assignment(assign);
            }
            ast::Stmt::Do(do_stmt) => {
                self.push_scope(ScopeKind::Block);
                self.visit_block(do_stmt.block());
                self.pop_scope();
            }
            ast::Stmt::If(if_stmt) => {
                self.visit_if(if_stmt);
            }
            ast::Stmt::NumericFor(for_stmt) => {
                self.visit_numeric_for(for_stmt);
            }
            ast::Stmt::GenericFor(for_stmt) => {
                self.visit_generic_for(for_stmt);
            }
            ast::Stmt::While(while_stmt) => {
                self.push_scope(ScopeKind::Loop);
                self.visit_block(while_stmt.block());
                self.pop_scope();
            }
            ast::Stmt::Repeat(repeat_stmt) => {
                self.push_scope(ScopeKind::Loop);
                self.visit_block(repeat_stmt.block());
                self.pop_scope();
            }
            _ => {
                // Other statements (FunctionCall, Return, etc.) — handle
                // expression-level analysis in future passes.
            }
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        // Register each declared name as a local variable
        for name_token in node.names() {
            let name = name_token.to_string().trim().to_string();
            let token_pos = name_token.start_position();
            let end_pos = name_token.end_position();
            let span = Span::new(
                Position::new(
                    token_pos.line() as u32 - 1,
                    token_pos.character() as u32 - 1,
                    0,
                ),
                Position::new(end_pos.line() as u32 - 1, end_pos.character() as u32 - 1, 0),
            );

            // Determine if this is a function assignment
            // (e.g., `local MyFunc = function() end`)
            let kind = SymbolKind::LocalVariable;
            // TODO: Check if the corresponding expression is a function
            // expression to classify as SymbolKind::Function

            self.define_symbol(name, kind, span);
        }
    }

    fn visit_function_declaration(&mut self, node: &ast::FunctionDeclaration) {
        // Global function declaration: function Name() end
        let name = node.name().to_string().trim().to_string();

        // Determine if it's a method (colon syntax)
        let is_method = name.contains(':');
        let kind = if is_method {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };

        // For the function name, use the position of the name token
        let span = Span::default(); // TODO: Extract proper span from name

        // Register as a global function
        self.define_symbol(name, kind, span);

        // Visit the function body in a new scope
        self.push_scope(ScopeKind::Function);

        // If it's a method, add implicit `self` parameter
        if is_method {
            let self_sym = Symbol {
                id: 0,
                name: "self".to_string(),
                kind: SymbolKind::Parameter,
                def_span: Span::default(),
                scope_id: self.current_scope(),
                uses: Vec::new(),
                is_parameter: true,
                is_implicit: true,
            };
            let id = self.symbols.add(self_sym);
            if let Some(scope) = self.scopes.get_mut(self.current_scope()) {
                scope.symbols.insert("self".to_string(), id);
            }
        }

        self.visit_block(node.body().block());
        self.pop_scope();
    }

    fn visit_local_function(&mut self, node: &ast::LocalFunction) {
        let name = node.name().to_string().trim().to_string();
        let token_pos = node.name().start_position();
        let end_pos = node.name().end_position();
        let span = Span::new(
            Position::new(
                token_pos.line() as u32 - 1,
                token_pos.character() as u32 - 1,
                0,
            ),
            Position::new(end_pos.line() as u32 - 1, end_pos.character() as u32 - 1, 0),
        );

        // Register as a local function
        self.define_symbol(name, SymbolKind::Function, span);

        // Visit the function body in a new scope
        self.push_scope(ScopeKind::Function);

        // Add parameters
        for param in node.body().parameters() {
            match param {
                ast::Parameter::Name(name_token) => {
                    let pname = name_token.to_string().trim().to_string();
                    self.define_symbol(pname, SymbolKind::Parameter, Span::default());
                }
                ast::Parameter::Ellipsis(_) => {
                    // Varargs — not a named symbol
                }
                _ => {}
            }
        }

        self.visit_block(node.body().block());
        self.pop_scope();
    }

    fn visit_assignment(&mut self, node: &ast::Assignment) {
        // Check for global assignments (names not previously defined locally)
        for var in node.variables() {
            match var {
                ast::Var::Name(name_token) => {
                    let name = name_token.to_string().trim().to_string();
                    // Check if this name is already defined locally
                    let is_local = self.scopes.lookup(&name, self.current_scope()).is_some();
                    if !is_local {
                        // This is a global variable assignment
                        let span = Span::default(); // TODO: extract span
                        self.define_symbol(name, SymbolKind::GlobalVariable, span);
                    }
                }
                _ => {
                    // Table field access, index — skip in v1
                }
            }
        }
    }

    fn visit_if(&mut self, node: &ast::If) {
        // If block
        self.push_scope(ScopeKind::Block);
        self.visit_block(node.block());
        self.pop_scope();

        // Elseif blocks
        if let Some(else_ifs) = node.else_if() {
            for else_if in else_ifs {
                self.push_scope(ScopeKind::Block);
                self.visit_block(else_if.block());
                self.pop_scope();
            }
        }

        // Else block
        if let Some(else_block) = node.else_block() {
            self.push_scope(ScopeKind::Block);
            self.visit_block(else_block);
            self.pop_scope();
        }
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        self.push_scope(ScopeKind::Loop);

        // Register loop variable
        let name = node.index_variable().to_string().trim().to_string();
        self.define_symbol(name, SymbolKind::LocalVariable, Span::default());

        self.visit_block(node.block());
        self.pop_scope();
    }

    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        self.push_scope(ScopeKind::Loop);

        // Register loop variables
        for name_token in node.names() {
            let name = name_token.to_string().trim().to_string();
            self.define_symbol(name, SymbolKind::LocalVariable, Span::default());
        }

        self.visit_block(node.block());
        self.pop_scope();
    }

    // Consuming accessor — returns all collected data at once.

    pub fn into_parts(self) -> (ScopeTree, SymbolTable, Vec<Diagnostic>) {
        (self.scopes, self.symbols, self.diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luagh_core::LineIndex;

    fn analyze_source(source: &str) -> Resolver<'static> {
        // We store the LineIndex in a leaked Box so we can pass a 'static ref.
        // This is fine for tests.
        let line_index = Box::leak(Box::new(LineIndex::new(source)));
        let mut resolver = Resolver::new(LuaVersion::Lua54, line_index);

        let ast = full_moon::parse(source).expect("test source should parse");
        let path = std::path::Path::new("test.lua");

        resolver.analyze(&ast, path);
        resolver
    }

    #[test]
    fn test_local_variable_registered() {
        let resolver = analyze_source("local x = 1");
        assert!(resolver.symbols.iter().any(|s| s.name == "x"));
    }

    #[test]
    fn test_local_function_registered() {
        let resolver = analyze_source("local function Foo() end");
        let sym = resolver
            .symbols
            .iter()
            .find(|s| s.name == "Foo")
            .expect("Foo should exist");
        assert_eq!(sym.kind, SymbolKind::Function);
    }

    #[test]
    fn test_scopes_created_for_function() {
        let resolver = analyze_source("local function Foo() local y = 2 end");
        // Module scope + function scope = at least 2 scopes
        assert!(resolver.scopes.len() >= 2);
    }
}
