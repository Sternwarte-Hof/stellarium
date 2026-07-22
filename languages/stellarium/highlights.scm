; Generated from tree-sitter-stellarium/queries/highlights.scm. Do not edit manually.

; Keywords
[
  "var"
  "let"
  "const"
  "function"
  "return"
  "if"
  "else"
  "for"
  "while"
  "do"
  "switch"
  "case"
  "default"
  "break"
  "continue"
  "try"
  "catch"
  "finally"
  "throw"
  "with"
  "new"
  "delete"
  "typeof"
  "void"
  "in"
  "instanceof"
] @keyword

; Boolean literals
(true) @boolean
(false) @boolean

; Null and undefined
(null) @constant.builtin
(undefined) @constant.builtin

; This keyword
(this) @variable.builtin

; Functions
(function_declaration
  name: (identifier) @function)

(function_expression
  name: (identifier) @function)

; Method definitions
(method_definition
  name: (property_identifier) @function.method)

; Function calls
(call_expression
  function: (identifier) @function.call)

(call_expression
  function: (member_expression
    property: (property_identifier) @function.method.call))

; Stellarium API objects (built-in globals)
(identifier) @variable

; Stellarium built-in functions - highlight include() calls
(call_expression
  function: (identifier) @function.builtin
  (#eq? @function.builtin "include"))

; Property identifiers
(property_identifier) @property

; Parameters
(formal_parameters
  (identifier) @variable.parameter)

(formal_parameters
  (pattern
    (identifier) @variable.parameter))

; Variables in declarations
(variable_declarator
  name: (identifier) @variable)

; Strings
(string
  (string_fragment) @string)

; Escape sequences
(escape_sequence) @string.escape

; Numbers
(number) @number

; Regex
(regex
  pattern: (regex_pattern) @string.regex
  flags: (regex_flags) @string.regex)

; Comments
(comment) @comment

; Binary operators
(binary_expression
  operator: _ @operator)

; Assignment operators
(augmented_assignment_expression
  operator: _ @operator)

; Update operators
(update_expression
  operator: _ @operator)

; Ternary operators
(ternary_expression
  "?" @operator
  ":" @operator)

; Punctuation
[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  "."
  ";"
] @punctuation.delimiter

; Assignment
(assignment_expression
  "=" @operator)

; Special: hash-bang line
(hash_bang_line) @comment

; Labels
(statement_identifier) @label

; Case clauses
(switch_case
  "case" @keyword)

(switch_default
  "default" @keyword)

; Getter/Setter
(method_definition
  "get" @keyword
  "set" @keyword)
