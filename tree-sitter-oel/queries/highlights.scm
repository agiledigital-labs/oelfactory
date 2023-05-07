; Special identifiers
;--------------------

; ([
;     (identifier)
;  ] @constant
;  (#match? @constant "^[A-Z_][A-Z\\d_]+$"))

((identifier) @variable.builtin
 (#match? @variable.builtin "^(user|appuser|idpuser|app|org|session)$"))

; Function and method calls
;--------------------------

; (call_expression
;   function: (identifier) @function)

; (call_expression
;   function: (member_expression
;     property: (property_identifier) @function.method))

; Variables
;----------

(identifier) @variable

; Properties
;-----------

; (property_identifier) @property

; Literals
;---------


[
  (true)
  (false)
  (null)
] @constant.builtin

[
  (string)
] @string

[(integer)(float)] @number

; Tokens
;-------

[
  "."
  ","
] @punctuation.delimiter

[
  "+"
  "<"
  "<="
  "=="
  "!"
  "!="
  ">"
  ">="
  "AND"
  "OR"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
]  @punctuation.bracket
