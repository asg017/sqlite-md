# `sqlite-md`

A SQLite extension for parsing, querying, and generating HTML from Markdown documents. Based on [markdown-rs](https://github.com/wooorm/markdown-rs) and [sqlite-loadable-rs](https://github.com/asg017/sqlite-loadable-rs).

Work in progress, not meant to be widely shared!

**Generate HTML from markdown**

```sql
select md_to_html('
# The largest heading

Paragraph with **some bold**, _underlines_, [links](https://google.com) and ![images](https://placeholder.com/assets/images/150x150-500x500.png)!

');
/*
'<h1>The largest heading</h1>
<p>Paragraph with <strong>some bold</strong>, <em>underlines</em>, <a href="https://google.com">links</a> and <img src="https://placeholder.com/assets/images/150x150-500x500.png" alt="images" />!</p>'
*/
```

**Extract all links from a markdown document**

```sql
select
  details ->> 'url' as url
from md_ast(readfile('README.md'))
where node_type = 'Link';
/*
┌──────────────────────────────────────────────┐
│                     url                      │
├──────────────────────────────────────────────┤
│ https://github.com/wooorm/markdown-rs        │
│ https://github.com/asg017/sqlite-loadable-rs │
└──────────────────────────────────────────────┘
*/
```

** Get all code samples from markdown**

```sql
select
  value as code,
  details ->> 'language' as language
from md_ast(readfile('README.md'))
where node_type = 'Code';

/*
┌──────────────────────────────────────────────────────────────┬──────────┐
│                             code                             │ language │
├──────────────────────────────────────────────────────────────┼──────────┤
│ select md_to_html('                                          │ sql      │
│ # The largest heading                                        │          │
│                                                              │          │
│ Paragraph with **some bold**, _underlines_, [links](https:// │          │
│ google.com) and ![images](https://placeholder.com/assets/ima │          │
│ ges/150x150-500x500.png)!                                    │          │
│                                                              │          │
│ ');                                                          │          │
│ /*                                                           │          │
│ '<h1>The largest heading</h1>                                │          │
│ <p>Paragraph with <strong>some bold</strong>, <em>underlines │          │
│ </em>, <a href="https://google.com">links</a> and <img src=" │          │
│ https://placeholder.com/assets/images/150x150-500x500.png" a │          │
│ lt="images" />!</p>'                                         │          │
│ *\/                                                          │          │
├──────────────────────────────────────────────────────────────┼──────────┤
│ select                                                       │ sql      │
│   details ->> 'url' as url                                   │          │
│ from md_ast(readfile('README.md'))                           │          │
│ where node_type = 'Link';                                    │          │
│ /*                                                           │          │
│ ┌──────────────────────────────────────────────┐             │          │
│ │                     url                      │             │          │
│ ├──────────────────────────────────────────────┤             │          │
│ │ https://github.com/wooorm/markdown-rs        │             │          │
│ │ https://github.com/asg017/sqlite-loadable-rs │             │          │
│ └──────────────────────────────────────────────┘             │          │
│ *\/                                                          │          │
├──────────────────────────────────────────────────────────────┼──────────┤
│ select                                                       │ sql      │
│   value as code,                                             │          │
│   details ->> 'language' as language                         │          │
│ from md_ast(readfile('README.md'))                           │          │
│ where node_type = 'Code';                                    │          │
└──────────────────────────────────────────────────────────────┴──────────┘
*/
```
