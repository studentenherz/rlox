Some notes on the development, places where I diverge from the book and stuff

- Taking inspiration from the lexer of the Rust cumpiler, in the lexer I didn't discard the whitespaces and comments. I don't have a use for them yet, but they might be useful for auto format and/or testing code inside comments.
- I also added support for multi-line block comments with /*...*/. To answer the questions in the 4th challenge in [Scanning](https://craftinginterpreters.com/scanning.html#challenges): I didn't think adding support for nesting was a lot useful, it wouldn't be dificult to implement, only need to keep track of opening and closing sequences, and check the balance.
- Giving it a second thought now, nesting comments could be useful. Let's say I have this code

  ```
  /** Comment 1 */ 
  fun func1() {
      print "hello";
  }
  ```

  with nesting comments I could comment all of that without having to modify the existing comment, something like this:

  ```
  /* Commenting this "temporarily"

  /** Comment 1 */ 
  fun func1() {
      print "hello";
  }

  */
  ```

  Right now this throws an error because there is no nesting support for comments.

- Implemented the comma operator, following the C operators precedence.
- Implemented the C-style "ternary" operator `?:`. These two weren't so hard to implement, but made following the book a little harder afterwards because the grammar definitions diverge. Not hard to do, just demmands a little more attention.
- Added error productions for binary operators without a left operand.
- Implemented the `+` operator for anything + string to concatenate.
- Didn't add an error for divide by zero. Rust's f64 gives `inf` when dividing by zero and then there are some operations with `inf` that give `inf` (like adding) and others that give `NaN` (like multiplying by 0). I think this is good enough and a sensible thing to get back from those mathematical operations.
- Added support for REPL to accept single expressions and evaluate instead of only statements. I'm not very happy with the way I'm handling it, but I think it's working for now.
- Made using uninitlized variables an error. I think this is better than silently using `nil` as default.
- Added `break` and `continue` statements. Also modified the `for` implementation to have an AST node instead of desugaring it in order to make the `continue` statement implementation easier.
