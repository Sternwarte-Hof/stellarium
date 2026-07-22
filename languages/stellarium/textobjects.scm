; Generated from tree-sitter-stellarium/queries/textobjects.scm. Do not edit manually.

; Textobjects for navigation and selection in Stellarium scripts

; Function definitions
(function_declaration
  (identifier) @function.name) @function.inside

(function_expression
  (identifier) @function.name) @function.inside

(function_declaration) @function.around

(function_expression) @function.around

; Parameters
(formal_parameters
  (_) @parameter.inside)

(formal_parameters) @parameter.around

; Arguments in function calls
(arguments
  (_) @parameter.inside)

(arguments) @parameter.around

; Comments
(comment) @comment.inside

(comment) @comment.around

; Blocks/statement blocks
(statement_block) @class.inside

; Objects
(object
  (_) @class.inside)

(object) @class.around

; Arrays
(array
  (_) @class.inside)

(array) @class.around

; If statements
(if_statement) @conditional.inside

; Loops
(for_statement) @loop.inside

(for_in_statement) @loop.inside

(while_statement) @loop.inside

(do_statement) @loop.inside

; Switch
(switch_statement) @conditional.inside

(switch_body
  (switch_case) @conditional.inside)

(switch_body
  (switch_default) @conditional.inside)

; Try-catch
(try_statement) @class.inside

(catch_clause) @class.inside

(finally_clause) @class.inside

; Variable declarations
(variable_declaration) @class.inside

; Return statements
(return_statement) @class.inside

; Expression statements
(expression_statement) @class.inside
