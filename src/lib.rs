mod ast;

use sqlite_loadable::{api, define_scalar_function, Result};
use sqlite_loadable::{define_table_function, prelude::*};

use crate::ast::MdAstTable;

pub fn md_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, format!("v{}", env!("CARGO_PKG_VERSION")))?;
    Ok(())
}

pub fn md_debug(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(
        context,
        format!(
            "Version: v{}
Source: {}
",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ),
    )?;
    Ok(())
}

pub fn md_to_html(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let md = api::value_text_notnull(values.get(0).expect("1st argument as name"))?;
    let html = markdown::to_html(md);
    api::result_text(context, html)?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_md_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "md_version", 0, md_version, flags)?;
    define_scalar_function(db, "md_debug", 0, md_debug, flags)?;
    define_scalar_function(db, "md_to_html", 1, md_to_html, flags)?;
    define_table_function::<MdAstTable>(db, "md_ast", None)?;
    Ok(())
}
