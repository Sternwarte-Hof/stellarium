; Generated from tree-sitter-stellarium/queries/locals.scm. Do not edit manually.

; Scopes

(program) @local.scope

(function_declaration) @local.scope

(function_expression) @local.scope

(statement_block) @local.scope

(catch_clause) @local.scope

; Definitions

(variable_declarator
  name: (identifier) @local.definition)

(function_declaration
  name: (identifier) @local.definition)

(formal_parameters
  (identifier) @local.definition)

; References
(identifier) @local.reference
