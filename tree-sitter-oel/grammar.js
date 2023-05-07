// Based on https://github.com/tree-sitter/tree-sitter-javascript/blob/master/grammar.js and trimmed down significantly

module.exports = grammar({
  name: "oel",

  precedences: ($) => [
    [
      "member",
      "call",
      "unary_void",
      "binary_concat",
      "binary_relation",
      "binary_equality",
      "bitwise_and",
      "bitwise_or",
      "logical_and",
      "logical_or",
      "ternary",
    ],
    ["member", "call", $.expression],
  ],

  //   inline: ($) => [$.expression],

  conflicts: ($) => [],

  rules: {
    source_file: ($) => $.expression,

    ternary_expression: ($) =>
      prec.right(
        "ternary",
        seq(
          // TODO: Conditions can't use Convert, Array, Time (probably validate in next stage)
          field("condition", $.expression),
          "?",
          field("consequence", $.expression),
          ":",
          field("alternative", $.expression)
        )
      ),

    expression: ($) =>
      choice(
        $.primary_expression,
        $.unary_expression,
        $.binary_expression,
        $.ternary_expression
      ),

    primary_expression: ($) =>
      choice(
        $.subscript_expression,
        $.member_expression,
        $.parenthesized_expression,
        $.identifier,
        $.primitive,
        $.array,
        $.call_expression
        // TODO: other kinds of expressions
      ),

    unary_expression: ($) =>
      prec.left(
        "unary_void",
        seq(field("operator", "!"), field("argument", $.expression))
      ),

    binary_expression: ($) =>
      choice(
        ...[
          ["AND", "logical_and"],
          ["OR", "logical_or"],
          ["+", "binary_concat"],
          ["<", "binary_relation"],
          ["<=", "binary_relation"],
          ["==", "binary_equality"],
          ["!=", "binary_equality"],
          [">=", "binary_relation"],
          [">", "binary_relation"],
        ].map(([operator, precedence, associativity]) =>
          (associativity === "right" ? prec.right : prec.left)(
            precedence,
            seq(
              field("left", $.expression),
              field("operator", operator),
              field("right", $.expression)
            )
          )
        )
      ),

    parenthesized_expression: ($) => seq("(", $.expression, ")"),

    member_expression: ($) =>
      prec(
        "member",
        seq(
          field("object", choice($.expression, $.primary_expression)),
          ".",
          field("property", alias($.identifier, "property_identifier"))
        )
      ),

    subscript_expression: ($) =>
      prec.right(
        "member",
        seq(
          field("object", choice($.expression, $.primary_expression)),
          "[",
          field("index", $.integer),
          "]"
        )
      ),

    arguments: ($) => seq("(", commaSep(optional($.expression)), ")"),

    nested_identifier: ($) =>
      prec(
        "member",
        seq(choice($.identifier, $.nested_identifier), ".", $.identifier)
      ),

    call_expression: ($) =>
      prec(
        "call",
        seq(field("function", $.expression), field("arguments", $.arguments))
      ),

    integer: ($) => /\d+/,
    // TODO: Should we use the proper IEEE spec here?
    float: ($) => /\d+\.\d+/,
    boolean: ($) => choice($.true, $.false),
    true: ($) => "true",
    false: ($) => "false",
    null: ($) => "null",
    array: ($) => seq("{", commaSep($.expression), "}"),

    // TODO: Can OEL have escape characters?
    string: ($) =>
      choice(
        seq(
          '"',
          repeat(alias($.unescaped_double_string_fragment, "string_fragment")),
          '"'
        ),
        seq(
          "'",
          repeat(alias($.unescaped_double_string_fragment, "string_fragment")),
          "'"
        )
      ),

    // Workaround to https://github.com/tree-sitter/tree-sitter/issues/1156
    // We give names to the token() constructs containing a regexp
    // so as to obtain a node in the CST.
    //
    unescaped_double_string_fragment: ($) =>
      token.immediate(prec(1, /[^"\\]+/)),

    identifier: ($) => {
      const alpha =
        /[^\x00-\x1F\s\p{Zs}0-9:;`"'@#.,|^&<=>+\-*/\\%?!~()\[\]{}\uFEFF\u2060\u200B]|\\u[0-9a-fA-F]{4}|\\u\{[0-9a-fA-F]+\}/;
      const alphanumeric =
        /[^\x00-\x1F\s\p{Zs}:;`"'@#.,|^&<=>+\-*/\\%?!~()\[\]{}\uFEFF\u2060\u200B]|\\u[0-9a-fA-F]{4}|\\u\{[0-9a-fA-F]+\}/;
      return token(seq(alpha, repeat(alphanumeric)));
    },

    primitive: ($) => choice($.boolean, $.null, $.float, $.integer, $.string),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)));
}

function commaSep(rule) {
  return optional(commaSep1(rule));
}
