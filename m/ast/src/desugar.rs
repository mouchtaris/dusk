//! Named parameter desugaring for `def` statements.
//!
//! # Motivation
//!
//! Currently, xs functions access their arguments via `$args[N]` indexing:
//!
//! ```dust
//! def documents::extern-section = {
//!     let title = $args[0];
//!     let base  = $args[1];
//!     let name  = $args[2];
//!     let path  = $args[3;];
//!     ...
//! }
//! ```
//!
//! This module enables a concise named-parameter syntax:
//!
//! ```dust
//! documents::extern-section title base name path[] = { ... }
//! ```
//!
//! The `[]` suffix marks a **rest parameter** (maps to `$args[N;]`).
//!
//! # Design
//!
//! This is a **pure AST-level desugaring** — no compiler, VM, or bytecode
//! changes required. The lalrpop grammar parses the new form and calls
//! [`desugar_def_params`] which synthesizes the equivalent `let` bindings
//! and prepends them to the function body. Everything downstream sees the
//! old AST shape.
//!
//! # Placement
//!
//! This file lives in `m/ast/src/desugar.rs` because it depends only on
//! AST types and factory functions (no parse or compile dependency).

use super::*;

// ---------------------------------------------------------------------------
// Static index strings for positional arg references.
// Avoids runtime allocation — these are `&'static str` which coerce to
// `&'i str` for any lifetime 'i.
// ---------------------------------------------------------------------------

const INDEX_STRS: [&str; 32] = [
    "0",  "1",  "2",  "3",  "4",  "5",  "6",  "7",
    "8",  "9",  "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23",
    "24", "25", "26", "27", "28", "29", "30", "31",
];

/// Retrieve the static index string for a given position.
/// Panics if `i >= 32` — enforce at parse time if needed.
fn index_str(i: usize) -> &'static str {
    INDEX_STRS[i]
}

// ---------------------------------------------------------------------------
// DefParam — the new AST node for named parameters
// ---------------------------------------------------------------------------

/// A named parameter in a `def` header.
///
/// ```text
/// def greet name greeting[] = { ... }
///           ^^^^            — Single("name")
///                ^^^^^^^^^^ — Rest("greeting")
/// ```
#[derive(Debug, Clone)]
pub enum DefParam<'i> {
    /// Positional: `name` → desugars to `let name = $args[N];`
    Single(&'i str),

    /// Rest / variadic: `name[]` → desugars to `let name = $args[N;];`
    Rest(&'i str),

    /// Const: `&name` → compile-time function address (default), not passed through $args
    Const(&'i str),

    /// Typed const: `&(name : type)` → compile-time constant with explicit type
    TypedConst(&'i str, &'i str),
}

impl<'i> DefParam<'i> {
    pub fn name(&self) -> &'i str {
        match self {
            DefParam::Single(n) | DefParam::Rest(n) | DefParam::Const(n)
            | DefParam::TypedConst(n, _) => n,
        }
    }

    pub fn is_rest(&self) -> bool {
        matches!(self, DefParam::Rest(_))
    }

    pub fn is_const(&self) -> bool {
        matches!(self, DefParam::Const(_) | DefParam::TypedConst(_, _))
    }

    pub fn const_param_type(&self) -> Option<ConstParamType> {
        match self {
            DefParam::Const(_) => Some(ConstParamType::Func),
            DefParam::TypedConst(_, typ) => Some(match *typ {
                "func" => ConstParamType::Func,
                "string" => ConstParamType::String,
                "number" => ConstParamType::Number,
                other => panic!("Unknown const param type: {}", other),
            }),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Core desugaring function
// ---------------------------------------------------------------------------

/// Desugar a `def` with named parameters into a standard `DefStmt`.
///
/// # Transformation
///
/// Given:
/// ```text
/// def my::func a b rest[] = { <user-body> }
/// ```
///
/// Produces the equivalent of:
/// ```text
/// def my::func = {
///     let a    = $args[0];
///     let b    = $args[1];
///     let rest = $args[2;];
///     <user-body>
/// }
/// ```
///
/// # Rules
///
/// - At most **one** rest parameter, and it **must be last**.
/// - Maximum 32 parameters (static index table limit).
/// - The rest parameter captures `$args[N;]` (from N to end).
///
/// # Panics
///
/// Panics if `params.len() > 32`. For a production build, convert
/// to a parse error instead (see integration notes in the diff).
pub fn desugar_def_params<'i>(
    name: Ident<'i>,
    params: Vec<DefParam<'i>>,
    body: Body<'i>,
) -> Item<'i> {
    assert!(
        params.len() <= INDEX_STRS.len(),
        "def {}: too many named parameters (max {})",
        name,
        INDEX_STRS.len()
    );

    // --- Validate: rest param may only appear as the last one ---
    for (i, p) in params.iter().enumerate() {
        if p.is_rest() && i != params.len() - 1 {
            panic!(
                "def {}: rest parameter `{}[]` must be the last parameter",
                name,
                p.name()
            );
        }
    }

    // --- Extract the existing body block ---
    let block = match body {
        Body::Block(b) => b,
    };
    let Block((existing_items, final_expr)) = block;

    // --- Build synthetic `let` bindings ---
    // Const params are skipped — they don't consume $args slots.
    let mut new_items: Vec<Item<'i>> =
        Vec::with_capacity(params.len() + existing_items.len());

    let mut runtime_idx: usize = 0;
    for param in params.iter() {
        if param.is_const() {
            // Const params don't get let bindings — they're resolved at the call site
            continue;
        }

        let idx: &str = index_str(runtime_idx);

        let (param_name, slice_expr) = match param {
            DefParam::Single(pname) => {
                // let pname = $args[N];
                //
                // AST: Expr::Slice(Slice(("args", Box(Range::Index(Natural("N"))))))
                let range = access_index(arg_nat(idx));
                let expr = Expr::Slice(Slice(("args", Box::new(range))));
                (*pname, expr)
            }
            DefParam::Rest(pname) => {
                // let pname = $args[N;];
                //
                // AST: Expr::Slice(Slice(("args", Box(Range::DoubleRange(
                //          Natural("N"), String("-0"))))))
                //
                // access_range with (Some(start), None) gives (start, "-0")
                // which is exactly the `$args[N;]` semantics (from N to end).
                let range = access_range((Some(arg_nat(idx)), None));
                let expr = Expr::Slice(Slice(("args", Box::new(range))));
                (*pname, expr)
            }
            DefParam::Const(_) | DefParam::TypedConst(_, _) => unreachable!(),
        };

        new_items.push(let_stmt(param_name, slice_expr));
        runtime_idx += 1;
    }

    // --- Append original body items ---
    new_items.extend(existing_items);

    // --- Reconstruct with the augmented block ---
    let new_block = Block((new_items, final_expr));
    let new_body = Body::Block(new_block);

    // --- If there are const params, emit TemplateDef instead of DefStmt ---
    let const_params: Vec<ConstParam<'i>> = params
        .iter()
        .filter_map(|p| {
            p.const_param_type().map(|typ| ConstParam {
                name: p.name(),
                typ,
            })
        })
        .collect();

    if const_params.is_empty() {
        Item::DefStmt(DefStmt((name, new_body)))
    } else {
        Item::TemplateDef(TemplateDef {
            name,
            body: new_body,
            const_params,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that desugaring a single positional param produces the right
    /// number of items.
    #[test]
    fn single_param_produces_one_let() {
        let body = Body::Block(Block((vec![], EXPR_0)));
        let params = vec![DefParam::Single("x")];

        match desugar_def_params("test", params, body) {
            Item::DefStmt(DefStmt((name, Body::Block(Block((items, _)))))) => {
                assert_eq!(name, "test");
                assert_eq!(items.len(), 1); // the synthesized `let x = $args[0];`
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    /// Verify rest param + positional params.
    #[test]
    fn mixed_params() {
        let body = Body::Block(Block((vec![], EXPR_0)));
        let params = vec![
            DefParam::Single("a"),
            DefParam::Single("b"),
            DefParam::Rest("rest"),
        ];

        match desugar_def_params("test", params, body) {
            Item::DefStmt(DefStmt((_, Body::Block(Block((items, _)))))) => {
                assert_eq!(items.len(), 3);
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    /// Verify that existing body items are preserved after the synthetic lets.
    #[test]
    fn preserves_existing_body() {
        let existing = vec![let_stmt("z", expr_nat("42"))];
        let body = Body::Block(Block((existing, EXPR_0)));
        let params = vec![DefParam::Single("x")];

        match desugar_def_params("test", params, body) {
            Item::DefStmt(DefStmt((_, Body::Block(Block((items, _)))))) => {
                // 1 synthetic + 1 existing
                assert_eq!(items.len(), 2);
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    #[should_panic(expected = "rest parameter")]
    fn rest_not_last_panics() {
        let body = Body::Block(Block((vec![], EXPR_0)));
        let params = vec![DefParam::Rest("bad"), DefParam::Single("after")];
        desugar_def_params("test", params, body);
    }
}
