#![feature(proc_macro, wasm_custom_section, wasm_import_module)]
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate log;
#[macro_use]
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

use pest::iterators::Pair;
use pest::Parser;
use serde_json::Value;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct JsonPathParser;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest");

type Node<'a> = Pair<'a, Rule>;

#[no_mangle]
#[wasm_bindgen]
pub extern "C" fn json_path(json: &str, path: &str) -> String {
  let value: Value = serde_json::from_str(json).unwrap();
  let mut runner = Runner::new(path).unwrap();
  let output = runner.run(value);
  serde_json::ser::to_string(&output).unwrap()
}

pub struct Runner<'a> {
  root: Value,
  ast: Node<'a>,
}

impl<'a> Runner<'a> {
  pub fn new(json_path: &'a str) -> Result<Self, String> {
    let mut ast =
      JsonPathParser::parse(Rule::json_path, json_path).map_err(|e| format!("{:?}", e))?;
    Ok(Self {
      root: Value::Null,
      ast: ast.next().unwrap(),
    })
  }

  pub fn run(&mut self, root: Value) -> Value {
    self.root = root;
    self.root(self.ast.clone()).clone()
  }

  fn filter_value(&self, context: &'a Value, filter_value: Node<'a>) -> Value {
    if filter_value.as_rule() != Rule::filter_value {
      unreachable!("Cannot run filter_value on node {:#?}", filter_value);
    }
    self.eval_path(context.clone(), filter_value)
  }

  fn parse_number(&self, expression: Node<'a>) -> Value {
    let string = expression.as_str().to_string();
    if let Ok(float) = string.parse::<f64>() {
      json!(float)
    } else if let Ok(p_int) = string.parse::<u64>() {
      json!(p_int)
    } else if let Ok(int) = string.parse::<i64>() {
      json!(int)
    } else {
      panic!("could not parse a json number value of {:#?}", string)
    }
  }

  fn filter_expression(&self, context: &Value, filter_expression: Node<'a>) -> bool {
    let mut iterator = filter_expression.into_inner();
    let left_expression = iterator.next().unwrap();
    let operator = iterator.next().unwrap();
    let right_expression = iterator.next().unwrap(); // Fixme not neccesarily a right value!
    let left_val = match left_expression.as_rule() {
      Rule::filter_value => self.filter_value(context, left_expression),
      _ => unimplemented!(
        "Unsupported filter expression left val of {:#?}",
        left_expression
      ),
    };
    let right_val = match right_expression.as_rule() {
      Rule::filter_value => self.filter_value(context, right_expression),
      Rule::word => json!(right_expression.as_str()),
      Rule::number => self.parse_number(right_expression),
      _ => unimplemented!(
        "Unsupported filter expression for right val of {:#?}",
        right_expression
      ),
    };

    match operator.as_rule() {
      Rule::filter_equal => left_val == right_val,
      Rule::filter_not_equal => left_val != right_val,
      Rule::filter_less => unimplemented!("Have not implemented filter operation of less"),
      Rule::filter_less_equal => {
        unimplemented!("Have not implemented filter operation of less equal")
      }
      Rule::filter_greater => unimplemented!("Have not implemented filter operation of greater"),
      Rule::filter_greater_equal => {
        unimplemented!("Have not implemented filter operation of greater equal")
      }
      Rule::filter_regex => unimplemented!("Have not implemented filter operation of regex"),
      Rule::filter_in => unimplemented!("Have not implemented filter operation of in"),
      Rule::filter_nin => unimplemented!("Have not implemented filter operation of nin"),
      Rule::filter_subsetof => unimplemented!("Have not implemented filter operation of subsetof"),
      Rule::filter_size => unimplemented!("Have not implemented filter operation of size"),
      Rule::filter_empty => unimplemented!("Have not implemented filter operation of emtpy"),
      _ => unreachable!("cannot reach filter operation for node of {:#?}", operator),
    }
  }

  fn should_filter(&self, context: &Value, filter: Node<'a>) -> bool {
    match filter.as_rule() {
      Rule::filter_value => self.filter_value(context, filter) != Value::Null,
      Rule::filter_expression => self.filter_expression(context, filter),
      _ => unimplemented!("not implemented filter for node of type {:#?}", filter),
    }
  }

  fn filter(&self, context: &Value, filter: Node<'a>) -> Value {
    let filter = filter.into_inner().next().unwrap();
    match context {
      &Value::Array(ref array) => {
        let mut filtered_array = vec![];
        for value in array {
          if self.should_filter(&value, filter.clone()) {
            filtered_array.push(value.clone());
          }
        }
        Value::Array(filtered_array)
      }
      _ => unreachable!("Cannot filter a non array value of {:#?}", context),
    }
  }

  /// Key can either be an expression or a word nothing else.
  fn key(&self, context: &Value, key: Node<'a>) -> Value {
    let key_value = key.into_inner().next().unwrap();
    match key_value.as_rule() {
      Rule::root_path => context[self.root(key_value).as_str().unwrap()].clone(),
      // FIXME does not handle cases of array values
      Rule::word => context[key_value.as_str()].clone(),
      Rule::filter => self.filter(context, key_value),
      _ => unimplemented!("unreachable key of node type {:#?}", key_value),
    }
  }

  fn root(&self, path: Node<'a>) -> Value {
    self.eval_path(self.root.clone(), path)
  }

  // Root run is an expression_run with context as root
  fn eval_path(&self, context: Value, path: Node<'a>) -> Value {
    let mut context = context;
    for node in path.into_inner() {
      match node.as_rule() {
        Rule::key => context = self.key(&context, node),
        _ => unreachable!("should not be able to get in with path node of {:#?}", node),
      };
    }
    context
  }
}

#[cfg(test)]
mod tests {
  fn test_json() -> Value {
    json!({
      "store": {
        "book": [
          {
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Old man and the sea",
            "price": 12.99
          },
          {
            "category": "fiction",
            "author": "Herman Melville",
            "title": "Moby Dick",
            "isbn": "0-553-21311-3",
            "price": 8.99
          },
          {
            "category": "fiction",
            "author": "J. R. R. Tolkien",
            "title": "The Lord of the Rings",
            "isbn": "0-395-19395-8",
            "price": 22.99
          }
        ],
        "bicycle": {
          "color": "red",
          "price": 19.95
        }
      },
      "expensive": 10
    })
  }

  use super::*;
  fn print_parse(rule: Rule, val: &str) {
    println!("{:#?}", JsonPathParser::parse(rule, val));
  }

  #[test]
  fn test_external_func() {
    assert_eq!(json_path("{\"blarg\":5,\"test\":5}", "$.blarg"), "5");
  }

  #[test]
  fn test_filter_expression_string() {
    let mut runner = Runner::new("$.store.book[?(@.category == 'reference')]").unwrap();
    assert_eq!(
      runner.run(test_json()),
      json!(
        [{
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          }
        ]
      )
    );
  }

  #[test]
  fn test_filter_expression() {
    let mut runner = Runner::new("$.store.book[?(@.price == 12.99)]").unwrap();
    assert_eq!(
      runner.run(test_json()),
      json!(
        [{
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Old man and the sea",
            "price": 12.99
          }
        ]
      )
    );
  }

  #[test]
  fn test_filter() {
    let value = json!({
      "test": {
        "array": [{"value": 5, "blarg": { "test": 15 }}, {"value": 7}, {}]
      }
    });
    let mut runner = Runner::new("$.test.array[?(@.blarg.test)]").unwrap();
    assert_eq!(
      runner.run(value),
      json!([{"value":5, "blarg": {"test" : 15}}])
    );
  }

  #[test]
  fn test_filter_expression_parse() {
    print_parse(Rule::key, "[?(@.value > @.test)]");
    parses_to!(
        parser: JsonPathParser,
        input: "[?(@.value > @.test)]",
        rule: Rule::key,
        tokens: [
          key(0, 21, [
            filter(1, 20, [
              filter_expression(3, 19, [
                filter_value(3, 10, [
                  key(4,10,[
                    word(5, 10)
                  ])
                ]),
                filter_greater(11, 12),
                filter_value(13, 19, [
                  key(14,19,[
                    word(15,19)
                  ])
                ])
              ])
            ])
          ])
        ]);
  }
  #[test]
  fn test_filter_parsing() {
    print_parse(Rule::key, "[?(@.value)]");
    parses_to!(
        parser: JsonPathParser,
        input: "[?(@.value)]",
        rule: Rule::key,
        tokens: [
          key(0, 12, [
            filter(1,11, [
              filter_value(3, 10, [
                key(4,10, [
                  word(5, 10)
                ])
              ])
            ])
          ])
        ]
      );
  }

  #[test]
  fn test_json_path_nesting() {
    let value = json!({
      "myKey": "key",
      "object": {
        "key": "SUCCESS"
      }
    });
    let mut runner = Runner::new("$.object[$.myKey]").unwrap();
    assert_eq!(json!("SUCCESS"), runner.run(value));
  }
  #[test]
  fn test_mixed_traversal() {
    let value = json!({
      "test": "blarg",
      "whatev": {
        "test": "hi!"
      }
    });
    let mut runner = Runner::new("$[\"whatev\"].test").unwrap();
    assert_eq!(runner.run(value.clone()), json!("hi!"));
    assert_eq!(runner.run(value.clone()), json!("hi!"));
  }
  #[test]
  fn test_obj_bracket_traversal() {
    let value = json!({
      "test": "blarg",
      "whatev": {
        "test": "hi!"
      }
    });
    let mut runner = Runner::new("$[\"whatev\"][\"test\"]").unwrap();
    assert_eq!(runner.run(value), json!("hi!"));
  }
  #[test]
  fn test_obj_dot_traversal() {
    let value = json!({
      "test": "blarg",
      "whatev": {
        "test": "hi!"
      }
    });
    let mut runner = Runner::new("$.whatev.test").unwrap();
    assert_eq!(runner.run(value), json!("hi!"));
  }
  #[test]
  fn true_lit() {
    parses_to!(
        parser: JsonPathParser,
        input: "true",
        rule: Rule::bool_lit,
        tokens: [
          bool_lit(0, 4, [
            true_lit(0,4)
          ])
        ]
      );
  }

  #[test]
  fn false_lit() {
    parses_to!(
          parser: JsonPathParser,
        input: "false",
        rule: Rule::bool_lit,
        tokens: [
          bool_lit(0, 5, [
            false_lit(0,5)
          ])
        ]
      );
  }

  #[test]
  fn digit_test() {
    parses_to!(
        parser: JsonPathParser,
        input: "9",
        rule: Rule::number,
        tokens: [
          number(0, 1, [
            int(0, 1, [
              digit(0, 1)
            ])
          ])
        ]
      );
    parses_to!(
        parser: JsonPathParser,
        input: "4",
        rule: Rule::number,
        tokens: [
          number(0, 1, [
            int(0, 1, [
              digit(0, 1)
            ])
          ])
        ]
      );
  }

  #[test]
  fn int_test() {
    parses_to!(
          parser: JsonPathParser,
          input: "123",
          rule: Rule::number,
          tokens: [
            number(0, 3, [
              int(0, 3, [
                digit(0,1),
                digit(1,2),
                digit(2,3),
              ])
            ])
          ]
        );
  }
  #[test]
  fn float_test() {
    parses_to!(
          parser: JsonPathParser,
          input: "123.123",
          rule: Rule::number,
          tokens: [
            number(0, 7, [
              int(0,3, [
                digit(0,1),
                digit(1,2),
                digit(2,3)
              ]),
              dot(3,4),
              int(4,7, [
                digit(4,5),
                digit(5,6),
                digit(6,7)
              ]),
            ])
          ]
        );
  }

  #[test]
  fn test_object_key() {
    parses_to!(
          parser: JsonPathParser,
          input: "te5",
          rule: Rule::word,
          tokens: [
            word(0,3)
          ]);
  }

  #[test]
  fn test_dot_notation() {
    parses_to!(
          parser: JsonPathParser,
          input: "$.test.123",
          rule: Rule::json_path,
          tokens: [
            root_path(0, 10, [
              key(1,6, [
                word(2,6)
              ]),
              key(6, 10, [
                word(7,10)
              ])
            ]),
          ]);
  }
  #[test]
  fn test_key_notation() {
    parses_to!(
          parser: JsonPathParser,
          input: "$[\"test\"][\"blarg\"]",
          rule: Rule::json_path,
          tokens: [
            root_path(0, 18, [
              key(1,9, [
                word(3,7)
              ]),
              key(9,18, [
                word(11,16)
              ])
            ]),
          ]);
  }
  #[test]
  fn test_key_notation_json_path() {
    parses_to!(
          parser: JsonPathParser,
          input: "$[\"test\"][$.test]",
          rule: Rule::json_path,
          tokens: [
            root_path(0, 17, [
              key(1,9, [
                word(3,7)
              ]),
              key(9, 17, [
                root_path(10,16, [
                  key(11,16, [
                    word(12, 16)
                  ])
                ])
              ])
            ]),
          ]);
  }
}
