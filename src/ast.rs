//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use markdown::mdast::Node;
use markdown::{to_mdast, ParseOptions};
use serde_json::json;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api, define_table_function,
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use std::{mem, os::raw::c_int};

fn all_nodes(root: Node) -> Vec<(i32, i32, Node)> {
    let mut nodes: Vec<(i32, i32, Node)> = vec![];
    let mut id = 0;
    let mut queue: Vec<(i32, i32, Node)> = vec![(0, 0, root)];
    while let Some(current) = queue.pop() {
        if let Some(children) = current.2.children() {
            for child in children {
                id += 1;
                //queue.push((id, current.0, child.clone()))
                queue.insert(0, (id, current.0, child.clone()))
            }
        }
        nodes.push(current);
    }

    nodes
}

static CREATE_SQL: &str = "CREATE TABLE x(parent, node_type, value, details, start_offset, start_line, start_column, end_offset, end_line, end_column, input_text hidden, raw hidden)";
enum Columns {
    Parent,
    NodeType,
    Value,
    Details,
    StartOffset,
    StartLine,
    StartColumn,
    EndOffset,
    EndLine,
    EndColumn,
    Raw,
    InputText,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Parent),
        1 => Some(Columns::NodeType),
        2 => Some(Columns::Value),
        3 => Some(Columns::Details),
        4 => Some(Columns::StartOffset),
        5 => Some(Columns::StartLine),
        6 => Some(Columns::StartColumn),
        7 => Some(Columns::EndOffset),
        8 => Some(Columns::EndLine),
        9 => Some(Columns::EndColumn),
        10 => Some(Columns::InputText),
        11 => Some(Columns::Raw),

        _ => None,
    }
}

#[repr(C)]
pub struct MdAstTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for MdAstTable {
    type Aux = ();
    type Cursor = MdAstCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, MdAstTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = MdAstTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_input = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::InputText) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_input = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(_) => (),
                None => todo!(),
            }
        }
        if !has_input {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);

        Ok(())
    }

    fn open(&mut self) -> Result<MdAstCursor> {
        Ok(MdAstCursor::new())
    }
}

#[repr(C)]
pub struct MdAstCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    all_nodes: Vec<(i32, i32, Node)>,
    input_text: String,
}
impl MdAstCursor {
    fn new() -> MdAstCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        MdAstCursor {
            base,
            rowid: 0,
            all_nodes: vec![],
            input_text: "".to_owned(),
        }
    }
}

impl VTabCursor for MdAstCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let input_text = api::value_text(values.get(0).unwrap())?;
        let options = ParseOptions::default();
        let root = to_mdast(input_text, &options)?;
        self.all_nodes = all_nodes(root);
        self.input_text = input_text.to_owned();
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.all_nodes.get(self.rowid as usize).is_none()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let (_id, parent, node) = self.all_nodes.get(self.rowid as usize).unwrap();
        match column(i) {
            Some(Columns::Parent) => api::result_int(context, *parent),
            Some(Columns::NodeType) => {
                let node_type = match node {
                    Node::BlockQuote(_) => "BlockQuote",
                    Node::Root(_) => "Root",
                    Node::FootnoteDefinition(_) => "FootnoteDefinition",
                    Node::MdxJsxFlowElement(_) => "MdxJsxFlowElement",
                    Node::List(_) => "List",
                    Node::MdxjsEsm(_) => "MdxjsEsm",
                    Node::Toml(_) => "Toml",
                    Node::Yaml(_) => "Yaml",
                    Node::Break(_) => "Break",
                    Node::InlineCode(_) => "InlineCode",
                    Node::InlineMath(_) => "InlineMath",
                    Node::Delete(_) => "Delete",
                    Node::Emphasis(_) => "Emphasis",
                    Node::MdxTextExpression(_) => "MdxTextExpression",
                    Node::FootnoteReference(_) => "FootnoteReference",
                    Node::Html(_) => "Html",
                    Node::Image(_) => "Image",
                    Node::ImageReference(_) => "ImageReference",
                    Node::MdxJsxTextElement(_) => "MdxJsxTextElement",
                    Node::Link(_) => "Link",
                    Node::LinkReference(_) => "LinkReference",
                    Node::Strong(_) => "Strong",
                    Node::Text(_) => "Text",
                    Node::Code(_) => "Code",
                    Node::Math(_) => "Math",
                    Node::MdxFlowExpression(_) => "MdxFlowExpression",
                    Node::Heading(_) => "Heading",
                    Node::Table(_) => "Table",
                    Node::ThematicBreak(_) => "ThematicBreak",
                    Node::TableRow(_) => "TableRow",
                    Node::TableCell(_) => "TableCell",
                    Node::ListItem(_) => "ListItem",
                    Node::Definition(_) => "Definition",
                    Node::Paragraph(_) => "Paragraph",
                };
                api::result_text(context, node_type)?
            }
            Some(Columns::Value) => {
                match node {
                    Node::Html(n) => api::result_text(context, n.value.as_str())?,
                    Node::Text(n) => api::result_text(context, n.value.as_str())?,
                    Node::Code(n) => api::result_text(context, n.value.as_str())?,
                    Node::Math(n) => api::result_text(context, n.value.as_str())?,
                    Node::InlineCode(n) => api::result_text(context, n.value.as_str())?,
                    Node::InlineMath(n) => api::result_text(context, n.value.as_str())?,
                    Node::Yaml(n) => api::result_text(context, n.value.as_str())?,
                    Node::Toml(n) => api::result_text(context, n.value.as_str())?,
                    Node::MdxjsEsm(n) => api::result_text(context, n.value.as_str())?,
                    Node::MdxFlowExpression(n) => api::result_text(context, n.value.as_str())?,
                    Node::MdxTextExpression(n) => api::result_text(context, n.value.as_str())?,
                    _ => (),
                };
            }
            Some(Columns::Details) => {
                let details: serde_json::Value = match node {
                    Node::Root(_) => serde_json::Value::Null,
                    Node::BlockQuote(_) => serde_json::Value::Null,
                    Node::FootnoteDefinition(_) => serde_json::Value::Null,
                    Node::MdxJsxFlowElement(_) => serde_json::Value::Null,
                    Node::List(list) => json!({
                      "ordered": list.ordered,
                      "start": list.start,
                      "spread": list.spread,
                    }),
                    Node::MdxjsEsm(_) => serde_json::Value::Null,
                    Node::Toml(_) => serde_json::Value::Null,
                    Node::Yaml(_) => serde_json::Value::Null,
                    Node::Break(_) => serde_json::Value::Null,
                    Node::InlineCode(_) => serde_json::Value::Null,
                    Node::InlineMath(_) => serde_json::Value::Null,
                    Node::Delete(_) => serde_json::Value::Null,
                    Node::Emphasis(_) => serde_json::Value::Null,
                    Node::MdxTextExpression(_) => serde_json::Value::Null,
                    Node::FootnoteReference(footnote_reference) => json!({
                      // TODO
                      "identifier": footnote_reference.identifier,
                      "label": footnote_reference.label,
                    }),
                    Node::Html(_) => serde_json::Value::Null,
                    Node::Image(image) => json!({
                      "alt": image.alt,
                      "url": image.url,
                      "title": image.title,
                    }),
                    Node::ImageReference(image_reference) => json!({
                      // TODO
                      //"reference_kind": image_reference.reference_kind,
                      "alt": image_reference.alt,
                      "identifier": image_reference.identifier,
                      "label": image_reference.label,
                    }),
                    Node::MdxJsxTextElement(_) => serde_json::Value::Null,
                    Node::Link(link) => json!({
                      "url": link.url,
                      "title": link.title,
                    }),
                    Node::LinkReference(link_reference) => json!({
                      // TODO
                      //"reference_kind": link_reference.reference_kind,
                      "identifier": link_reference.identifier,
                      "label": link_reference.label,
                    }),
                    Node::Strong(_) => serde_json::Value::Null,
                    Node::Text(_) => serde_json::Value::Null,
                    Node::Code(code) => json!({
                      "language": code.lang,
                      "meta": code.meta,
                    }),
                    Node::Math(math) => json!({
                      "meta": math.meta,
                    }),
                    Node::MdxFlowExpression(_) => serde_json::Value::Null,
                    Node::Heading(heading) => json!({"depth": heading.depth}),
                    Node::Table(_) => json!({
                        // TODO
                        //"align": table.align,
                    }),
                    Node::ThematicBreak(_) => serde_json::Value::Null,
                    Node::TableRow(_) => serde_json::Value::Null,
                    Node::TableCell(_) => serde_json::Value::Null,
                    Node::ListItem(list_item) => json!({
                      "spread": list_item.spread,
                      "checked": list_item.checked,
                    }),
                    Node::Definition(definition) => json!({
                      "url": definition.url,
                      "title": definition.title,
                      "identifier": definition.identifier,
                      "label": definition.label,
                    }),
                    Node::Paragraph(_) => serde_json::Value::Null,
                };
                if details == serde_json::Value::Null {
                    api::result_null(context);
                } else {
                    api::result_json(context, details)?;
                }
            }
            Some(Columns::StartOffset) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.start.offset as i64),
                );
            }
            Some(Columns::StartLine) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.start.line as i64),
                );
            }
            Some(Columns::StartColumn) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.start.column as i64),
                );
            }
            Some(Columns::EndOffset) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.end.offset as i64),
                );
            }
            Some(Columns::EndLine) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.end.line as i64),
                );
            }
            Some(Columns::EndColumn) => {
                node.position().map_or_else(
                    || api::result_null(context),
                    |p| api::result_int64(context, p.end.column as i64),
                );
            }

            Some(Columns::Raw) => node.position().map_or_else(
                || api::result_null(context),
                |p| {
                    api::result_text(
                        context,
                        self.input_text[p.start.offset..p.end.offset].to_string(),
                    )
                    .unwrap()
                },
            ),
            // TODO
            Some(Columns::InputText) => (),
            None => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.all_nodes.get(self.rowid as usize).unwrap().0 as i64)
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_seriesrs_init(db: *mut sqlite3) -> Result<()> {
    define_table_function::<MdAstTable>(db, "generate_series_rs", None)?;
    Ok(())
}
