//! Displaying IR.

use crate::ir::*;
use crate::module::functions::{Function, FunctionKind, ImportedFunction, LocalFunction};
use id_arena::Id;
use std::mem;

/// A trait for displaying our parsed IR.
pub trait DisplayIr {
    /// Extra context needed to display this thing.
    type Context;

    /// Display this IR into the given formatter.
    fn display_ir(&self, f: &mut String, ctx: &Self::Context, indent: usize);
}

impl DisplayIr for Function {
    type Context = ();

    fn display_ir(&self, f: &mut String, _: &(), indent: usize) {
        assert_eq!(indent, 0);
        match self.kind {
            FunctionKind::Import(ref i) => i.display_ir(f, &(), indent),
            FunctionKind::Local(ref l) => l.display_ir(f, &(), indent),
            FunctionKind::Uninitialized(_) => unreachable!(),
        }
    }
}

impl DisplayIr for ImportedFunction {
    type Context = ();

    fn display_ir(&self, f: &mut String, _: &(), indent: usize) {
        assert_eq!(indent, 0);
        f.push_str("(import func)");
    }
}

impl DisplayIr for LocalFunction {
    type Context = ();

    fn display_ir(&self, f: &mut String, _: &(), indent: usize) {
        assert_eq!(indent, 0);

        let mut visitor = DisplayExpr {
            func: self,
            f,
            indent,
            first_arg: false,
            line: 0,
        };
        // leading spaces to leave room for leading expression ids
        visitor.f.push_str("        (func\n");
        self.entry_block().visit(&mut visitor);
        visitor.f.push_str("        )");
    }
}

pub(crate) struct DisplayExpr<'a, 'b> {
    pub(crate) func: &'a LocalFunction,
    pub(crate) f: &'b mut String,
    indent: usize,
    first_arg: bool,
    line: usize,
}

impl DisplayExpr<'_, '_> {
    // Prints the index of ids such as memories, locals, globals, etc.
    pub(crate) fn id<T>(&mut self, id: Id<T>) {
        self.f.push_str(" ");
        self.f.push_str(&id.index().to_string());
    }

    fn line(&mut self) {
        self.line += 1;
        self.f.push_str("\n");
    }

    pub(crate) fn expr_id(&mut self, id: ExprId) {

        // If we're the first argument of a previous expression, then we start
        // ourselves on a new line. Otherwise we're already starting on a line.
        let first_arg = mem::replace(&mut self.first_arg, true);
        if first_arg {
            self.line();
        }

        // Start all lines with the id of this expression, used as a handy
        // debugging reference.
        self.f.push_str("(;");
        self.f.push_str(&format!("{:3}", id.index()));
        self.f.push_str(";)");

        // Start the s-expression of this expression with a properly indented
        // parentheses. After recursing, which prints everything, we push a new
        // line if we had any arguments. All instructions end with a newline
        // afterwards too!
        self.indent += 1;
        self.indent();
        self.f.push_str("(");
        let start = self.line;
        id.visit(self);
        if start != self.line {
            self.f.push_str("       ");
            self.indent();
        }
        self.indent -= 1;
        self.f.push_str(")");
        self.line();
        self.first_arg = false;
    }

    fn indent(&mut self) {
        self.f.push_str(" ");
        for _ in 0..self.indent {
            self.f.push_str("  ");
        }
    }
}

// Note that the main body of `DisplayExpr` is generated by `#[walrus_expr]`
