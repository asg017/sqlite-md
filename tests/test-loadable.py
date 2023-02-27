import sqlite3
import unittest
import time
import os

EXT_PATH="./dist/debug/md0"

def connect(ext):
  db = sqlite3.connect(":memory:")

  db.execute("create table base_functions as select name from pragma_function_list")
  db.execute("create table base_modules as select name from pragma_module_list")

  db.enable_load_extension(True)
  db.load_extension(ext)

  db.execute("create temp table loaded_functions as select name from pragma_function_list where name not in (select name from base_functions) order by name")
  db.execute("create temp table loaded_modules as select name from pragma_module_list where name not in (select name from base_modules) order by name")

  db.row_factory = sqlite3.Row
  return db


db = connect(EXT_PATH)

def explain_query_plan(sql):
  return db.execute("explain query plan " + sql).fetchone()["detail"]

def execute_all(sql, args=None):
  if args is None: args = []
  results = db.execute(sql, args).fetchall()
  return list(map(lambda x: dict(x), results))

FUNCTIONS = [
  "md_debug",
  "md_to_html",
  "md_version",
]

MODULES = [
  "md_ast",
]
def spread_args(args):                                                          
  return ",".join(['?'] * len(args))

class TestMd(unittest.TestCase):
  def test_funcs(self):
    funcs = list(map(lambda a: a[0], db.execute("select name from loaded_functions").fetchall()))
    self.assertEqual(funcs, FUNCTIONS)

  def test_modules(self):
    modules = list(map(lambda a: a[0], db.execute("select name from loaded_modules").fetchall()))
    self.assertEqual(modules, MODULES)
    
  def test_md_version(self):
    self.assertEqual(db.execute("select md_version()").fetchone()[0][0], "v")
  
  def test_md_debug(self):
    debug = db.execute("select md_debug()").fetchone()[0]
    self.assertEqual(len(debug.splitlines()), 2)
  
  def test_md_to_html(self):
    md_to_html = lambda md: db.execute("select md_to_html(?)", [md]).fetchone()[0]
    self.assertEqual(md_to_html('**bold**'), '<p><strong>bold</strong></p>')
    self.assertEqual(md_to_html('[Documentation](#docs)'), '<p><a href="#docs">Documentation</a></p>')
  
  def test_md_ast(self):
    md_ast = lambda content: execute_all("select rowid, *, raw from md_ast(?)", [content])

    self.assertEqual(
      md_ast('alex **garcia** [yo](#yoyo)'),
      [
      {'rowid': 0, 'parent': 0, 'node_type': 'Root',      'value': None,      'details': None,                            'start_offset': 0, 'start_line': 1, 'start_column': 1, 'end_offset': 27, 'end_line': 1, 'end_column': 28, 'raw': 'alex **garcia** [yo](#yoyo)'}, 
      {'rowid': 1, 'parent': 0, 'node_type': 'Paragraph', 'value': None,      'details': None,                            'start_offset': 0, 'start_line': 1, 'start_column': 1, 'end_offset': 27, 'end_line': 1, 'end_column': 28, 'raw': 'alex **garcia** [yo](#yoyo)'}, 
      {'rowid': 2, 'parent': 1, 'node_type': 'Text',      'value': 'alex ',   'details': None,                            'start_offset': 0, 'start_line': 1, 'start_column': 1, 'end_offset': 5, 'end_line': 1, 'end_column': 6, 'raw': 'alex '}, 
      {'rowid': 3, 'parent': 1, 'node_type': 'Strong',    'value': None,      'details': None,                            'start_offset': 5, 'start_line': 1, 'start_column': 6, 'end_offset': 15, 'end_line': 1, 'end_column': 16, 'raw': '**garcia**'}, 
      {'rowid': 4, 'parent': 1, 'node_type': 'Text',      'value': ' ',       'details': None,                            'start_offset': 15, 'start_line': 1, 'start_column': 16, 'end_offset': 16, 'end_line': 1, 'end_column': 17, 'raw': ' '}, 
      {'rowid': 5, 'parent': 1, 'node_type': 'Link',      'value': None,      'details': '{"title":null,"url":"#yoyo"}',  'start_offset': 16, 'start_line': 1, 'start_column': 17, 'end_offset': 27, 'end_line': 1, 'end_column': 28, 'raw': '[yo](#yoyo)'}, 
      {'rowid': 6, 'parent': 3, 'node_type': 'Text',      'value': 'garcia',  'details': None,                            'start_offset': 7, 'start_line': 1, 'start_column': 8, 'end_offset': 13, 'end_line': 1, 'end_column': 14, 'raw': 'garcia'}, 
      {'rowid': 7, 'parent': 5, 'node_type': 'Text',      'value': 'yo',      'details': None,                            'start_offset': 17, 'start_line': 1, 'start_column': 18, 'end_offset': 19, 'end_line': 1, 'end_column': 20, 'raw': 'yo'}]
    )

  
class TestCoverage(unittest.TestCase):                                      
  def test_coverage(self):                                                      
    test_methods = [method for method in dir(TestMd) if method.startswith('test_')]
    funcs_with_tests = set([x.replace("test_", "") for x in test_methods])
    
    for func in FUNCTIONS:
      self.assertTrue(func in funcs_with_tests, f"{func} does not have corresponding test in {funcs_with_tests}")
    
    for module in MODULES:
      self.assertTrue(module in funcs_with_tests, f"{module} does not have corresponding test in {funcs_with_tests}")

if __name__ == '__main__':
    unittest.main()