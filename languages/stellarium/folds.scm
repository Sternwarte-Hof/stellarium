; Generated from tree-sitter-stellarium/queries/folds.scm. Do not edit manually.

; Folding rules for Stellarium scripts

; Fold function bodies
(function_declaration
  body: (statement_block) @fold)

(function_expression
  body: (statement_block) @fold)

(method_definition
  body: (statement_block) @fold)

; Fold control flow blocks
(if_statement
  consequence: (statement_block) @fold)

(if_statement
  alternative: (else_clause
    (statement_block) @fold))

(while_statement
  body: (statement_block) @fold)

(do_statement
  body: (statement_block) @fold)

(for_statement
  body: (statement_block) @fold)

(for_in_statement
  body: (statement_block) @fold)

; Fold try-catch-finally
(try_statement
  body: (statement_block) @fold)

(catch_clause
  body: (statement_block) @fold)

(finally_clause
  body: (statement_block) @fold)

; Fold switch bodies
(switch_body) @fold

; Fold object literals
(object) @fold

; Fold array literals (multi-line arrays)
(array) @fold

; Fold block statements
(statement_block) @fold
