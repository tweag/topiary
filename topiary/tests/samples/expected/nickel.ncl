{
  regression_tests = [
    # Interpolated record operation chains
    {
      # This is fine
      good = "%{interpolated}",

      # An interpolated (record_operation_chain) breaks the spacing
      bad = "%{interpolated.string}"
    }
  ],

  # Nickel standard library as of af0d5ee

  array = {
    NonEmpty
      | doc m%"
        Contract to ensure the given array is not empty.

        For example:
        ```nickel
          ([] | NonEmpty) =>
            error
          ([ 1 ] | NonEmpty) =>
            [ 1 ]
        ```
        "%
      = fun label value =>
        if %typeof% value == `Array then
          if %length% value != 0 then
            value
          else
            %blame% (%label_with_message% "empty array" label)
        else
          %blame% (%label_with_message% "not a array" label),

    first
      : forall a. Array a -> a
      | doc m%"
        Results in the first element of the given array.

        # Examples

        ```nickel
          first [ "this is the head", "this is not" ] =>
            "this is the head"
        ```
        "%
      = fun array => %elem_at% array 0,

    last
      : forall a. Array a -> a
      | doc m%"
        Results in the last element of the given array.

        # Examples

        ```nickel
          last [ "this is the head", "this is not" ] =>
            "this is not"
        ```
        "%
      = fun array => %elem_at% array (%length% array - 1),

    drop_first
      : forall a. Array a -> Array a
      | doc m%"
        Results in the given array without the first element.

        For example:
        ```nickel
          drop_first [ 1, 2, 3 ] =>
            [ 2, 3 ]
        ```
        "%
      = fun array => %array_slice% 1 (%length% array) array,

    drop_last
      : forall a. Array a -> Array a
      | doc m%"
        Results in the given array without the last element.

        For example:
        ```nickel
          drop_last [ 1, 2, 3 ] =>
            [ 1, 2 ]
        ```
        "%
      = fun array => %array_slice% 0 (%length% array - 1) array,

    length
      : forall a. Array a -> Number
      | doc m%"
        Results in a number representing the length of the given array.

        For example:
        ```nickel
          length [ "Hello,", " World!" ] =>
            2
        ```
        "%
      = fun l => %length% l,

    map
      : forall a b. (a -> b) -> Array a -> Array b
      | doc m%"
        `map f [x1, x2, ..., xn]` applies function `f` to every element in the array,
        resulting in `[f x1, f x2, ... f xn]`

        For example:
        ```nickel
          map (fun x => x + 1) [ 1, 2, 3 ] =>
            [ 2, 3, 4 ]
        ```
        "%
      = fun f l => %map% l f,

    at
      : forall a. Number -> Array a -> a
      | doc m%"
        Retrieves the n'th element from a array (0-indexed).

        For example:
        ```nickel
          at 3 [ "zero" "one" "two" "three" "four" ] =>
            "three"
        ```
        "%
      = fun n l => %elem_at% l n,

    concat
      : forall a. Array a -> Array a -> Array a
      | doc m%"
        Concatenates two arrays such that the second array is appended to the first.

        For example:
        ```nickel
          concat [ 1, 2, 3 ] [ 4, 5, 6 ] =>
            [ 1, 2, 3, 4, 5, 6 ]
        ```
        "%
      = fun l1 l2 => l1 @ l2,

    fold_left
      : forall a b. (a -> b -> a) -> a -> Array b -> a
      | doc m%"
        Fold a function over an array. Folds serve a similar purpose to loops or
        iterators in a functional language like Nickel. `fold_left` iterates over
        an array, by repeatedly applying a function over each element, threading
        an additional arbitrary state (the accumulator, of type `a` in the
        signature).

        `fold_left f init [x1, x2, ..., xn]` results in `f (... (f (f init x1) x2) ...) xn`.

        This function is strict in the intermediate accumulator.

        # Left vs right

        Folds come in two variants, left and right. How to decide which one to
        use?

        - If the folded function isn't associative (such as subtraction), then
          each variant will give a different result. The choice is dictacted by
          which one you need. For example:

          ```nickel
          array.fold_right (-) 0 [1, 2, 3, 4]
           => -2
          array.fold_left (-) 0 [1, 2, 3, 4]
           => -10
          ```
        - If the folded function is associative, both `fold_right` and
          `fold_left` return the same result. In that case, **`fold_left` is
          generally preferred**, because it forces the evaluation of the
          intermediate results resulting in less memory consumption and overall
          better performance (outside of pathological cases).
          `fold_left` also iterates from the start of the array, which
          correponds to the usual behavior of loops and iterators in most
          programming languages. There is one case where `fold_right` might be
          preferred, see the next point.
        - If the folded function is associative but _(left) short-circuiting_,
          meaning that it can sometimes determine the result without using the
          right argument, then `fold_right` provides early return. An example is
          the boolean AND operator `&&`: when evaluating `left && right`, if
          `left` is `false`, the whole expression will evaluate to `false`
          without even evaluating `right`. Consider the following expression:

          ```nickel
          array.replicate 1000 true
          # gives [false, .. true 1000 times]
          |> array.prepend false
          |> array.fold_right (&&) [false]
          ```

          Here, `fold_right` will stop at the first element, and the operation
          runs in constant time, given the definition of `fold_right` and the
          lazy evaluation of Nickel. If we had used `fold_left` instead, which
          is closer to a standard iterator, we would have iterated over all of
          the 1000 elements of the array.

        # Examples

        ```nickel
          fold_left (fun acc e => acc + e) 0 [ 1, 2, 3 ] =>
            (((0 + 1) + 2) 3) =>
            6
        ```
        "%
      = fun f acc l =>
        let length = %length% l in
        if length == 0 then
          acc
        else
          let rec go =
            fun acc n =>
              if n == length then
                acc
              else
                let next_acc = %elem_at% l n |> f acc in
                %seq% next_acc (go next_acc (n + 1))
          in
          go acc 0,

    fold_right
      : forall a b. (a -> b -> b) -> b -> Array a -> b
      | doc m%"
        Fold a function over an array. Folds serve a similar purpose to loops or
        iterators in a functional language like Nickel. `fold_right` iterates
        over an array, by repeatedly applying a function to each element and
        threading an additional arbitrary state (the accumulator of type `a` in
        the signature).

        `fold_right f init [x1, x2, ..., xn]` results in `f x1 (f x2 (... (f xn init) ...))`.

        # Left vs right

        Folds come in two variants, left and right. How to decide which one to
        use? Please refer to the documentation of `fold_left`.

        # Examples

        ```nickel
          fold_right (fun e acc => acc @ [e]) [] [ 1, 2, 3 ] =>
            ((([] @ [3]) @ [2]) @ [1]) =>
            [ 3, 2, 1 ]
        ```
        "%
      = fun f fst l =>
        let length = %length% l in
        let rec go =
          fun n =>
            if n == length then
              fst
            else
              go (n + 1)
              |> f (%elem_at% l n)
        in go 0,

    prepend
      : forall a. a -> Array a -> Array a
      | doc m%"
        Construct an array given the first element and the rest of the array.

        For example:
        ```nickel
          prepend 1 [ 2, 3 ] =>
            [ 1, 2, 3 ]
        ```
        "%
      = fun x l => [x] @ l,

    append
      : forall a. a -> Array a -> Array a
      | doc m%"
        Construct an array given the last element and the rest of the array.

        For example:
        ```nickel
          append 3 [ 1, 2 ] =>
            [ 1, 2, 3 ]
        ```
        "%
      = fun x l => l @ [x],

    reverse
      : forall a. Array a -> Array a
      | doc m%"
        Reverses the order of a array.

        For example:
        ```nickel
          reverse [ 1, 2, 3 ] =>
            [ 3, 2, 1 ]
        ```
        "%
      = fun l => fold_left (fun acc e => [e] @ acc) [] l,

    filter
      : forall a. (a -> Bool) -> Array a -> Array a
      | doc m%"
        `filter f xs` keeps all elements from `xs` given that satisfy `f`.

        For example:
        ```nickel
          filter (fun x => x <= 3) [ 4, 3, 2, 5, 1 ] =>
            [ 3, 2, 1 ]
        ```
        "%
      = fun pred l => fold_left (fun acc x => if pred x then acc @ [x] else acc) [] l,

    flatten
      : forall a. Array (Array a) -> Array a
      | doc m%"
        Flatten a array of arrays to a single array, essentially concatenating all arrays in the original array.

        For example:
        ```nickel
          flatten [[1, 2], [3, 4]] =>
            [1, 2, 3, 4]
        ```
        "%
      = fun l => fold_right (fun l acc => l @ acc) [] l,

    all
      : forall a. (a -> Bool) -> Array a -> Bool
      | doc m%"
        Results in true if all elements in the given array satisfy the predicate, false otherwise.

        For example:
        ```nickel
          all (fun x => x < 3) [ 1, 2 ] =>
            true
          all (fun x => x < 3) [ 1, 2 3 ] =>
            false
        ```
        "%
      = fun pred l => fold_right (fun x acc => if pred x then acc else false) true l,

    any
      : forall a. (a -> Bool) -> Array a -> Bool
      | doc m%"
        Results in false if no elements in the given array satisfy the predicate, true otherwise.

        For example:
        ```nickel
          any (fun x => x < 3) [ 1, 2, 3, 4 ] =>
            true
          any (fun x => x < 3) [ 5, 6, 7, 8 ] =>
            false
        ```
        "%
      = fun pred l => fold_right (fun x acc => if pred x then true else acc) false l,

    elem
      : forall a. a -> Array a -> Bool
      | doc m%"
        Results in true if the given element is a member of the array, false otherwise.

        For example:
        ```nickel
          elem 3 [ 1, 2, 3, 4, 5 ] =>
            true
        ```
        "%
      = fun elt => any (fun x => x == elt),

    partition
      : forall a. (a -> Bool) -> Array a -> { right : Array a, wrong : Array a }
      | doc m%"
        Partitions the given array in two new arrays: those containing the elements that satisfy the predicate, and those
        that do not.

        For example:
        ```nickel
          partition (fun x => x < 5) [ 2, 4, 5, 3, 7, 8, 6 ] =>
            { right = [ 3, 4, 2 ], wrong = [ 6, 8, 7, 5 ] }
        ```
        "%
      = fun pred l =>
        let aux =
          fun acc x =>
            if (pred x) then
              { right = acc.right @ [x], wrong = acc.wrong }
            else
              { right = acc.right, wrong = acc.wrong @ [x] }
        in
        fold_left aux { right = [], wrong = [] } l,

    generate
      : forall a. (Number -> a) -> Number -> Array a
      | doc m%"
        `generate f n` produces a array of length `n` by applying `f` on increasing numbers:
         `[ f 0, f 1, ..., f (n - 1) ]`.

        For example:
        ```nickel
          generate function.id 4 =>
            [ 0, 1, 2, 3 ]
        ```
        "%
      = fun f n => %generate% n f,

    sort
      | forall a. (a -> a -> [| `Lesser, `Equal, `Greater |]) -> Array a -> Array a
      | doc m%"
        Sorts the given arrays based on the provided comparison operator.

        For example:
        ```nickel
          sort (fun x y => if x < y then `Lesser else if (x == y) then `Equal else `Greater) [ 4, 5, 1, 2 ] =>
            [ 1, 2, 4, 5 ]
        ```
        "%
      #TODO: maybe inline partition to avoid contract checks?
      = fun cmp array =>
        let length = %length% array in
        let first = %elem_at% array 0 in
        let rest = %array_slice% 1 length array in
        let parts = partition (fun x => (cmp x first == `Lesser)) rest in
        if length <= 1 then
          array
        else
          (sort cmp (parts.right)) @ [first] @ (sort cmp (parts.wrong)),

    flat_map
      : forall a b. (a -> Array b) -> Array a -> Array b
      | doc m%"
        First maps the given function over the array and then flattens the
        result.

        For example:
        ```nickel
        flat_map (fun x => [x, x]) [1, 2, 3]
          => [1, 1, 2, 2, 3, 3]
        ```
      "%
      = fun f xs => map f xs |> flatten,

    intersperse
      : forall a. a -> Array a -> Array a
      | doc m%"
        Intersperse the passed value between the elements of an array.

        For example:
        ```nickel
        intersperse ", " [ "Hello", "wonderful", "world!" ]
          => [ "Hello", ", ", "wonderful", ", ", "world!" ]
        intersperse ", " [ "Hello" ]
          => [ "Hello" ]
        intersperse ", " []
          => []
        ```
      "%
      = fun v array =>
        let length = %length% array in
        if length <= 1 then
          array
        else
          let first = %elem_at% array 0 in
          let rest = %array_slice% 1 length array in
          [first] @ (flat_map (fun a => [ v, a ]) rest),

    slice
      : forall a. Number -> Number -> Array a -> Array a
      | doc m%"
          `slice start end array` returns the slice of `array` between `start` (included) and
          `end` (excluded).

          # Preconditions

          In `slice start end value`, `start` and `end` must be positive
          integers such that `0 <= start <= end <= array.length value`.

          # Examples

          ```nickel
          slice 1 3 [ 0, 1, 2, 3, 4, 5]
            => [ 1, 2 ]
          slice 0 3 [ "Hello", "world", "!" ]
            => [ "Hello", "world", "!" ]
          slice 2 3 [ "Hello", "world", "!" ]
            => [ "!" ]
           ```
        "%
      = fun start end value => %array_slice% start end value,

    split_at
      : forall a. Number -> Array a -> { left : Array a, right : Array a }
      | doc m%"
          Splits an array in two at a given index, and puts all the elements
          to the left of the element at the given index (excluded) in the
          `left` field, and the rest of the array in the `right` field.

          # Preconditions

          In `split_at inded value`, `index` must be a positive integer such
          that `0 <= index <= array.length value`.

          # Examples

          ```nickel
          split_at 2 [ 0, 1, 2, 3, 4, 5]
            => { left = [ 2, 3, 4, 5 ], right = [ 0, 1 ] }
          split_at 0 [ "Hello", "world", "!" ]
            => { left = [  ], right = [ "Hello", "world", "!" ] }
          split_at 3 [ "Hello", "world", "!" ]
            => { left = [ "Hello", "world", "!" ], right = [  ] }
          ```
        "%
      = fun index value =>
        {
          left = %array_slice% 0 index value,
          right = %array_slice% index (%length% value) value
        },

    replicate
      : forall a. Number -> a -> Array a
      | doc m%"
          `replicate n x` creates an array containing `x` exactly `n` times.

          # Preconditions

          `n` must be an integer greater or equal to `0`.

          # Examples

          ```nickel
          replicate 0 false
            => [ ]
          replicate 5 "x"
            => [ "x", "x", "x", "x", "x" ]
          ```
        "%
      = fun n x => %generate% n (fun _i => x),

    range_step
      : Number -> Number -> Number -> Array Number
      | doc m%"
          `range_step start end step` generates the array of numbers
          `[start, start + step, start + 2*step, ..]` up to the first element
          (excluded) larger than or equal to `end`

          # Preconditions

          In `range_step start end step`, `start` and `end` must satisfy `start
          <= end`. `step` must be strictly greater than `0`.

          # Examples

          ```nickel
          range_step (-1.5) 2 0.5
           => [ -1.5, -1, -0.5, 0, 0.5, 1, 1.5 ]
          ```
        "%
      = fun start end step =>
        %generate% ((end - start) / step |> number.floor) (fun i => start + i * step),

    range
      : Number -> Number -> Array Number
      | doc m%"
          `range start end` generates the array of numbers
          `[start, start + 1, start + 2, ..]` up to the first element
          (excluded) larger than or equal to `end`.

          `range start end` is equivalent to `range_step start end 1`.

          # Preconditions

          In `range_step start end`, `start` and `end` must satisfy
          `start <= end`.

          # Examples

          ```nickel
          range 0 5
           => [ 0, 1, 2, 3, 4 ]
          ```
        "%
      = fun start end => range_step start end 1,

    reduce_left
      : forall a. (a -> a -> a) -> Array a -> a
      | doc m%"
        Reduces the elements to a single one, by repeatedly applying a reducing
        operation. If the array is empty, returns an empty array.

        `reduce_left` associates to the left, that is
        `reduce_left op [x1, x2, ..., xn]` results in `op (... (op (op x1 x2) x3) ...) xn`.

        `reduce_left` is the same as `fold_left`, but uses the first element as
        the initial accumulator.

        # Left vs right

        The rationale to decide between `fold_left` and `fold_right` applies to
        `reduce_left` and `reduce_right` as well. See the documentation of
        `fold_left`.

        # Examples

        ```nickel
          reduce_left (@) [ [1, 2], [3], [4,5] ]
            => (([1, 2] @ [3]) @ [4,5])
            => [ 1, 2, 4, 5 ]
          reduce_left (-) [ 1, 2, 3, 4]
            => ((1 - 2) - 3) - 4
            => -8
        ```
        "%
      = fun f array =>
        let first = %elem_at% array 0 in
        let rest = %array_slice% 1 (%length% array) array in
        fold_left f first rest,

    reduce_right
      : forall a. (a -> a -> a) -> Array a -> a
      | doc m%"
        Reduces the elements to a single one, by repeatedly applying a reducing
        operation. If the array is empty, returns an empty array.

        `reduce_right` associates to the right, that is
        `reduce_right op [x1, x2, ..., xn]` results in
        `op x1 (op x2 (... (op xn-1 xn) ...))`.

        `reduce_right` is the same as `fold_right`, but uses the last element as
        the initial element.

        # Left vs right

        The rationale to decide between `fold_left` and `fold_right` applies to
        `reduce_left` and `reduce_right` as well. See the documentation of
        `fold_left`.

        # Examples

        ```nickel
          reduce_right (@) [ [1, 2], [3], [4,5] ]
            => [1, 2] @ ([3] @ [4,5])
            => [ 1, 2, 4, 5 ]
          reduce_right (-) [ 1, 2, 3, 4]
            => 1 - (2 - (3 - 4))
            => -2
        ```
        "%
      = fun f array =>
        let last_index = %length% array - 1 in
        let last = %elem_at% array last_index in
        let rest = %array_slice% 0 last_index array in
        fold_right f last rest,
  },

  builtin = {
    is_number
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a number.

      For example:
      ```nickel
        is_number 1 =>
          true
        is_number "Hello, World!" =>
          false
      ```
      "%
      = fun x => %typeof% x == `Number,

    is_bool
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a boolean.

      For example:
      ```nickel
        is_bool false =>
          true
        is_bool 42 =>
          false
      ```
      "%
      = fun x => %typeof% x == `Bool,

    is_string
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a string.

      For example:
      ```nickel
        is_string true =>
          false
        is_string "Hello, World!" =>
          true
      ```
      "%
      = fun x => %typeof% x == `String,

    is_enum
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is an enum tag.

      For example:
      ```nickel
        is_enum true =>
          false
        is_enum `false =>
          true
      ```
      "%
      = fun x => %typeof% x == `Enum,

    is_function
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a function.

      For example
      ```nickel
        is_function (fun x => x) =>
          true
        is_function 42 =>
          false
      ```
      "%
      = fun x => %typeof% x == `Function,

    is_array
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a array.

      For example
      ```nickel
        is_array [ 1, 2 ] =>
          true
        is_array 42 =>
          false
      ```
      "%
      = fun x => %typeof% x == `Array,

    is_record
      : Dyn -> Bool
      | doc m%"
      Checks if the given value is a record.

      For example
      ```nickel
        is_record [ 1, 2 ] =>
          false
        is_record { hello = "Hello", world = "World" } =>
          true
      ```
      "%
      = fun x => %typeof% x == `Record,

    typeof
      : Dyn -> [|
        `Number,
        `Bool,
        `String,
        `Enum,
        `Label,
        `Function,
        `Array,
        `Record,
        `Other
      |]
      | doc m%"
      Results in a value representing the type of the typed value.

      For example:
      ```nickel
        typeof [ 1, 2 ] =>
          `Array
        typeof (fun x => x) =>
          `Function
      ```
      "%
      = fun x => %typeof% x,

    seq
      : forall a. Dyn -> a -> a
      | doc m%"
      `seq x y` forces the evaluation of `x`, before resulting in `y`.

      For example:
      ```nickel
        seq (42 / 0) 37 =>
          error
        seq (42 / 2) 37 =>
          37
        seq { tooFar = 42 / 0 } 37 =>
          37
      ```
      "%
      = fun x y => %seq% x y,

    deep_seq
      : forall a. Dyn -> a -> a
      | doc m%"
      `deep_seq x y` forces a deep evaluation `x`, before resulting in `y`.

      For example:
      ```nickel
        deep_seq (42 / 0) 37 =>
          error
        deep_seq (42 / 2) 37 =>
          37
        deep_seq { tooFar = 42 / 0 } 37 =>
          error
      ```
      "%
      = fun x y => %deep_seq% x y,

    hash
      : [| `Md5, `Sha1, `Sha256, `Sha512 |] -> String -> String
      | doc m%"
      Hashes the given string provided the desired hash algorithm.

      For example:
      ```nickel
        hash `Md5 "hunter2" =>
          "2ab96390c7dbe3439de74d0c9b0b1767"
      ```
      "%
      = fun type s => %hash% type s,

    serialize
      : [| `Json, `Toml, `Yaml |] -> Dyn -> String
      | doc m%"
      Serializes the given value to the desired representation.

      For example:
      ```nickel
        serialize `Json { hello = "Hello", world = "World" } =>
          "{
            "hello": "Hello",
            "world": "World"
          }"
      ```
      "%
      = fun format x => %serialize% format (%force% x),

    deserialize
      : [| `Json, `Toml, `Yaml |] -> String -> Dyn
      | doc m%"
      Deserializes the given string to a nickel value given the encoding of the string.

      For example:
      ```nickel
        deserialize `Json "{ \"hello\": \"Hello\", \"world\": \"World\" }"
          { hello = "Hello", world = "World" }
      ```
      "%
      = fun format x => %deserialize% format x,

    to_string
      | string.Stringable -> String
      | doc m%"
      Converts a stringable value to a string representation. Same as
      `string.from`.

      For example:
      ```nickel
      from 42 =>
        "42"
      from `Foo =>
        "Foo"
      from null =>
        "null"
      ```
      "%
      = fun x => %to_str% x,

    trace
      : forall a. String -> a -> a
      | doc m%"
      `builtin.trace msg x` prints `msg` to standard output when encountered by the evaluator,
      and proceed with the evaluation of `x`.

      For example:
      ```nickel
      builtin.trace "Hello, world!" true =>
        builtin.trace: Hello, world!
        true
      ```
      "%
      = fun msg x => %trace% msg x,

    FailWith
      | doc m%"
      A contract that always fails with a given message.

      For example:
      ```nickel
      1 | FailWith "message" =>
        error: contract broken by a value: message
          ┌─ :1:1
          │
        1 │ FailWith "message"
          │ ---------------------------- expected type
          │
          ┌─ repl-input-0:1:1
          │
        1 │ 1 | FailWith "message"
          │ ^ applied to this expression
      ```
    "%
      = fun msg label value => contract.blame_with_message msg label,

    fail_with
      | String -> Dyn
      | doc m%"
      Unconditionally abort evaluation with the given message.

      For example:
      ```nickel
      fail_with "message" =>
        error: contract broken by a value: message
      ```
    "%
      = fun msg => null | FailWith msg,
  },

  contract = {
    blame
      | doc m%"
        Raise blame for a given label.

        Type: `forall a. Lbl -> a`
        (for technical reasons, this function isn't actually statically typed)

        Blame is the mechanism to signal contract violiation in Nickel. It ends
        the program execution and print a detailed report thanks to the
        information tracked inside the label.

        For example:

        ```nickel
        IsZero = fun label value =>
          if value == 0 then
            value
          else
            contract.blame label
        ```
        "%
      = fun label => %blame% label,

    blame_with_message
      | doc m%"
        Raise blame with respect to a given label and a custom error message.

        Type: `forall a. String -> Lbl -> a`
        (for technical reasons, this function isn't actually statically typed)

        Same as `blame`, but take an additional custom error message that will be
        displayed as part of the blame error. `blame_with_message message label`
        is equivalent to `blame (label.with_message message label)`

        For example:

        ```nickel
        let IsZero = fun label value =>
          if value == 0 then
            value
          else
            contract.blame_with_message_message "Not zero" label
        in

        0 | IsZero
        ```
        "%
      = fun message label => %blame% (%label_with_message% message label),

    from_predicate
      | doc m%"
        Generate a contract from a boolean predicate.

        Type: `(Dyn -> Bool) -> (Lbl -> Dyn -> Dyn)`
        (for technical reasons, this function isn't actually statically typed)

        For example:

        ```nickel
        let IsZero = contract.from_predicate (fun x => x == 0) in
        0 | IsZero
        ```
        "%
      = fun pred label value => if pred value then value else %blame% label,

    label
      | doc m%"
          The label submodule, which gathers functions that manipulate the label
          of a contract.

          A label is a special opaque value automatically passed by the Nickel
          interpreter to contracts when performing a contract check.

          A label stores a stack of custom error diagnostics, that can be
          manipulated by the function of this module. Labels thus offer a way to
          customize the error message that will be shown if the contract is broken.

          The setters (`with_XXX` functions) always operate on the current error
          diagnostic, which is the last diagnotic on the stack (if the stack is
          empty, a fresh diagnostic is silently created when using a setter).
          The previous diagnostics are thus archived, and can't be modified
          anymore. All diagnostics will be shown during error reporting, with
          the most recent being used as the main one.

          `contract.apply` is the operation that pushes a new fresh diagnostic on
          the stack, saving the previously current error diagnostic. Indeed,
          `contract.apply` is mostly used to apply subcontracts inside a parent
          contract. Stacking the current diagnostic potentially customized by
          the parent contract saves the information inside, and provides a fresh
          diagnostic for the child contract to use.
        "%
      = {
        with_message
          | doc m%"
            Attach a custom error message to the current error diagnostic of a
            label.

            Type: `String -> Lbl -> Lbl`
            (for technical reasons, this function isn't actually statically typed)

            If a custom error message was previously set, there are two
            possibilities:
              - the label has gone through a `contract.apply` call in-between.
                In this case, the previous diagnostic has been stacked,
                and using `with_message` won't erase anything but rather modify
                the fresh current diagnostic.
              - no `contract.apply` has taken place since the last message was
                set. In this case, the current diagnostic is still the same, and
                the previous error message will be erased.

            For example:

            ```nickel
            let ContractNum = contract.from_predicate (fun x => x > 0 && x < 50) in

            let Contract = fun label value =>
              if builtin.is_number value then
                contract.apply
                  ContractNum
                  (contract.label.with_message
                    "num subcontract failed! (out of bound)"
                    label
                  )
                  value
              else
                value
            in

            5 | Contract
            ```
            "%
          = fun message label => %label_with_message% message label,

        with_notes
          | doc m%"
            Attach custom error notes to the current error diagnostic of a
            label.

            Type: `Array String -> Lbl -> Lbl`
            (for technical reasons, this function isn't actually statically typed)

            If custom error notes were previously set, there are two
            possibilities:
              - the label has gone through a `contract.apply` call in-between.
                In this case, the previous diagnostic has been stacked,
                and using `with_notes` won't erase anything but rather modify
                the fresh current diagnostic.
              - no `contract.apply` has taken place since the last message was
                set. In this case, the current diagnostic is still the same, and
                the previous error notes will be erased.

            For example:

            ```nickel
            let ContractNum = contract.from_predicate (fun x => x > 0 && x < 50) in

            let Contract = fun label value =>
              if builtin.is_number value then
                contract.apply
                  ContractNum
                  (label
                   |> contract.label.with_message "num subcontract failed! (out of bound)"
                   |> contract.label.with_notes [
                        "The value was a number, but this number is out of the expected bound",
                        "The value must be a number between 0 and 50, both excluded",
                      ]
                  )
                  value
              else
                value
            in

            5 | Contract
            ```
            "%
          # the %label_with_notes% operator expects an array of strings which is
          # fully evaluated, thus we force the notes first
          = fun notes label => %label_with_notes% (%force% notes) label,

        append_note
          | doc m%"
              Append a note to the notes of the current diagnostic of a label.

              Type: `String -> Lbl -> Lbl`
              (for technical reasons, this function isn't actually statically typed)
            "%
          = fun note label => %label_append_note% note label,
      },

    apply
      | doc m%"
          Apply a contract to a label and a value.

          Type: `Contract -> Lbl -> Dyn -> Dyn`
          (for technical reasons, this function isn't actually statically typed)

          Nickel supports user-defined contracts defined as functions, but also
          as records. Moreover, the interpreter performs additional book-keeping
          for error reporting when applying a contract in an expression
          `value | Contract`. You should not use standard function application
          to apply a contract, but this function instead.

          # Example

          ```nickel
          let Nullable = fun param_contract label value =>
            if value == null then null
            else contract.apply param_contract label value
          in
          let Contract = Nullable {foo | Number} in
          ({foo = 1} | Contract)
          ```

          # Diagnostic stack

          Using `apply` will stack the current custom reporting data, and create a
          fresh current working diagnostic. `apply` thus acts automatically as a
          split point between a contract and its subcontracts, providing the
          subcontracts with a fresh diagnostic to use, while remembering the
          previous diagnostics set by parent contracts.

          ## Illustration

          ```nickel
          let ChildContract = fun label value =>
            label
            |> contract.label.with_message "child's message"
            |> contract.label.append_note "child's note"
            |> contract.blame
          in

          let ParentContract = fun label value =>
            let label =
              label
              |> contract.label.with_message "parent's message"
              |> contract.label.append_note "parent's note"
            in
            contract.apply ChildContract label value
          in

          null | ParentContract
          ```

          This example will print two diagnostics: the main one, using the
          message and note of the child contract, and a secondary diagnostic,
          using the parent contract message and note.
        "%
      = fun contract label value =>
        %assume% contract (%label_push_diag% label) value,
  },

  enum = {
    Tag
      | doc m%"
          Contract to enforce the value is an enum tag.

          # Examples

          ```nickel
            (`foo | Tag) =>
              `foo
            (`FooBar | Tag) =>
              `FooBar
            ("tag" | Tag) =>
              error
          ```
          "%
      = contract.from_predicate builtin.is_enum,

    TagOrString
      | doc m%%"
            Accepts both enum tags and strings. Strings are automatically
            converted to an enum tag.

            `TagOrString` is typically used in conjunction with an enum type, to
            accept tags represented as strings (e.g. coming from a JSON
            serialization) as well.

            # Examples

            ``` nickel
            let Schema = {
              protocol
                | enum.TagOrString
                | [| `http, `ftp |],
              port
                | Number,
              method
                | enum.TagOrString
                | [| `GET, `POST |]
            } in
            let serialized =
              m%"
                {"protocol": "http", "port": 80, "method": "GET"}
              "%
              |> builtin.deserialize `Json
            in

            serialized | Schema
            ```
          "%%
      = fun label value =>
        %typeof% value
        |> match {
          `String => %enum_from_str% value,
          `Enum => value,
          _ =>
            contract.blame_with_message "expected either a string or an enum tag" label,
        },
  },

  function = {
    id
      : forall a. a -> a
      | doc m%"
    The identity function, that is
    ```nickel
    id x == x
    ```
    for any value `x`.
    "%
      = fun x => x,

    const
      : forall a b. a -> b -> a
      | doc m%"
    Results in the first argument.

    For example:
    ```nickel
    const 5 42 == 5
    ```
    "%
      = fun x y => x,

    compose
      : forall a b c. (b -> c) -> (a -> b) -> (a -> c)
      | doc m%"
    Function composition, right to left. That is,
    ```nickel
    compose f g x == f (g x)
    ```
    "%
      = fun g f x => x |> f |> g,

    flip
      : forall a b c. (a -> b -> c) -> b -> a -> c
      | doc m%%"
    Flip the argument order for a two-argument function. For example,
    ```nickel
    flip (fun x y => "%{x} %{y}") "world!" "Hello,"
      => "Hello, world!"
    ```
    "%%
      = fun f x y => f y x,

    first
      : forall a b. a -> b -> a
      | doc m%"
    Always returns the first argument:
    ```nickel
    first 5 7 => 5
    ```
    "%
      = const,

    second
      : forall a b. a -> b -> b
      | doc m%"
    Always returns the second argument:
    ```nickel
    second 5 7 => 7
    ```
    "%
      = flip const,

    pipe
      : forall a. a -> Array (a -> a) -> a
      | doc m%%"
      Apply an array of functions to a value, in order.

      For example:
      ```nickel
      pipe 2 [ (+) 2, (+) 3 ]
        => 7
      pipe `World [ string.from, fun s => "Hello, %{s}!" ]
        => "Hello, World!"
      ```
    "%%
      = fun x fs => array.fold_left (|>) x fs,
  },

  # Internal operations. Can't be accessed from user code because `$` is not a
  # valid starting character for an identifier.

  # Contract implementations
  "$dyn" = fun _label value => value,

  "$num" = fun label value => if %typeof% value == `Number then value else %blame% label,

  "$bool" = fun label value => if %typeof% value == `Bool then value else %blame% label,

  "$string" = fun label value => if %typeof% value == `String then value else %blame% label,

  "$fail" = fun label _value => %blame% label,

  "$array" = fun element_contract label value =>
    if %typeof% value == `Array then
      %array_lazy_assume% (%go_array% label) value element_contract
    else
      %blame% label,

  "$func" = fun domain codomain label value =>
    if %typeof% value == `Function then
      (fun x => %assume% codomain (%go_codom% label) (value (%assume% domain (%chng_pol% (%go_dom% label)) x)))
    else
      %blame% label,

  "$forall_var" = fun sealing_key label value =>
    let current_polarity = %polarity% label in
    let polarity = (%lookup_type_variable% sealing_key label).polarity in
    if polarity == current_polarity then
      %unseal% sealing_key value (%blame% label)
    else
      # Here, we know that this term should be sealed, but to give the right
      # blame for the contract, we have to change the polarity to match the
      # polarity of the `Forall`, because this is what's important for
      # blaming polymorphic contracts.
      %seal% sealing_key (%chng_pol% label) value,

  "$forall" = fun sealing_key polarity contract label value =>
    contract (%insert_type_variable% sealing_key polarity label) value,

  "$enums" = fun case label value =>
    if %typeof% value == `Enum then
      %assume% case label value
    else
      %blame% (%label_with_message% "not an enum tag" label),

  "$enum_fail" = fun label =>
    %blame% (%label_with_message% "tag not included in the enum type" label),

  "$record" = fun field_contracts tail_contract label value =>
    if %typeof% value == `Record then
      # Returns the sub-record of `l` containing only those fields which are not
      # present in `r`. If `l` has a sealed polymorphic tail then it will be
      # preserved.
      let field_diff =
        fun left right =>
          array.fold_left (fun acc field =>
            if %has_field% field right then
              acc
            else
              %record_insert% field acc (left."%{field}")) (%record_empty_with_tail% left) (%fields% left)
      in
      let contracts_not_in_value = field_diff field_contracts value in
      let missing_fields = %fields% contracts_not_in_value in
      if %length% missing_fields == 0 then
        let tail_fields = field_diff value field_contracts in
        let fields_with_contracts =
          array.fold_left (fun acc field =>
            if %has_field% field field_contracts then
              let contract = field_contracts."%{field}" in
              let label = %go_field% field label in
              let val = value."%{field}" in
              %record_insert% field acc (%assume% contract label val)
            else
              acc) {} (%fields% value)
        in
        tail_contract fields_with_contracts label tail_fields
      else
        let plural = if %length% missing_fields == 1 then "" else "s" in
        %blame% (%label_with_message% "missing field%{plural} `%{string.join ", " missing_fields}`" label)
    else
      %blame% (%label_with_message% "not a record" label),

  "$dyn_record" = fun contract label value =>
    if %typeof% value == `Record then
      %dictionary_assume% (%go_dict% label) value contract
    else
      %blame% (%label_with_message% "not a record" label),

  "$forall_tail" = fun sealing_key acc label value =>
    let current_polarity = %polarity% label in
    let polarity = (%lookup_type_variable% sealing_key label).polarity in
    if polarity == current_polarity then
      if value == {} then
        let tagged_label = %label_with_message% "polymorphic tail mismatch" label in
        let tail = %record_unseal_tail% sealing_key tagged_label value in
        acc & tail
      else
        let extra_fields = %fields% value in
        let plural = if %length% extra_fields == 1 then "" else "s" in
        %blame% (%label_with_message% "extra field%{plural} `%{string.join ", " extra_fields}`" label)
    else
      # Note: in order to correctly attribute blame, the polarity of `l`
      # must match the polarity of the `forall` which introduced the
      # polymorphic contract (i.e. `pol`). Since we know in this branch
      # that `pol` and `%polarity% l` differ, we swap `l`'s polarity before
      # we continue.
      %record_seal_tail% sealing_key (%chng_pol% label) acc value,

  "$dyn_tail" = fun acc label value => acc & value,

  "$empty_tail" = fun acc label value =>
    if value == {} then
      acc
    else
      let extra_fields = %fields% value in
      let plural = if %length% extra_fields == 1 then "" else "s" in
      %blame% (%label_with_message% "extra field%{plural} `%{string.join ", " extra_fields}`" label),

  # Recursive priorities operators

  "$rec_force" = fun value => %rec_force% (%force% value),
  "$rec_default" = fun value => %rec_default% (%force% value),

  number = {
    Integer
      | doc m%"
      Contract to enforce a number is an integer.

      For example:
      ```nickel
        (1.5 | Integer) =>
          error
        (42 | Integer) =>
          42
      ```
      "%
      = fun label value =>
        if %typeof% value == `Number then
          if value % 1 == 0 then
            value
          else
            %blame% (%label_with_message% "not an integer" label)
        else
          %blame% (%label_with_message% "not a number" label),

    Nat
      | doc m%"
      Contract to enforce a number is a natural number (including 0).

      For example:
      ```nickel
        (42 | Nat) =>
          42
        (0 | Nat) =>
          0
        (-4 | Nat) =>
          error
      ```
      "%
      = fun label value =>
        if %typeof% value == `Number then
          if value % 1 == 0 && value >= 0 then
            value
          else
            %blame% (%label_with_message% "not a natural" label)
        else
          %blame% (%label_with_message% "not a number" label),

    PosNat
      | doc m%"
      Contract to enforce a number is a positive natural number.

      For example:
      ```nickel
        (42 | PosNat) =>
          42
        (0 | PosNat) =>
          error
        (-4 | PosNat) =>
          error
      ```
      "%
      = fun label value =>
        if %typeof% value == `Number then
          if value % 1 == 0 && value > 0 then
            value
          else
            %blame% (%label_with_message% "not positive integer" label)
        else
          %blame% (%label_with_message% "not a number" label),

    NonZero
      | doc m%"
      Contract to enforce a number is anything but zero.

      For example:
      ```nickel
        (1 | NonZero) =>
          1
        (0.0 | NonZero) =>
          error
      ```
      "%
      = fun label value =>
        if %typeof% value == `Number then
          if value != 0 then
            value
          else
            %blame% (%label_with_message% "non-zero" label)
        else
          %blame% (%label_with_message% "not a number" label),

    is_integer
      : Number -> Bool
      | doc m%"
      Checks if the given number is an integer.

      For example:
      ```nickel
        is_int 42 =>
          true
        is_int 1.5 =>
          false
      ```
      "%
      = fun x => x % 1 == 0,

    min
      : Number -> Number -> Number
      | doc m%"
      Results in the lowest of the given two numbers.

      For example:
      ```nickel
        min (-1337) 42 =>
          -1337
      ```
      "%
      = fun x y => if x <= y then x else y,

    max
      : Number -> Number -> Number
      | doc m%"
      Results in the highest of the given two numbers.

      For example:
      ```nickel
        max (-1337) 42 =>
          42
      ```
      "%
      = fun x y => if x >= y then x else y,

    floor
      : Number -> Number
      | doc m%"
      Rounds the number down to the next integer.

      For example:
      ```nickel
        floor 42.5 =>
          42
      ```
      "%
      = fun x =>
        if x >= 0 then
          x - (x % 1)
        else
          x - 1 - (x % 1),

    abs
      : Number -> Number
      | doc m%"
      Results in the absolute value of the given number.

      For example:
      ```nickel
        abs (-5) =>
          5
        abs 42 =>
          42
      ```
      "%
      = fun x => if x < 0 then -x else x,

    fract
      : Number -> Number
      | doc m%"
      Results in the fractional part of the given number.

      For example:
      ```nickel
        fract 13.37 =>
          0.37
        fract 42 =>
          0
      ```
      "%
      = fun x => x % 1,

    truncate
      : Number -> Number
      | doc m%"
      Truncates the given number.

      For example:
      ```nickel
        truncate (-13.37) =>
          -13
        truncate 42.5 =>
          42
      ```
      "%
      = fun x => x - (x % 1),

    pow
      : Number -> Number -> Number
      | doc m%"
      `pow x y` results in `x` to the power of `y`.

      For example:
      ```nickel
        pow 2 8 =>
          256
      ```

      # Precision

      Nickel numbers are arbitrary precision rationals. If the exponent `y` is
      an integer which fits in a 64-bits signed or unsigned integer (that is, if
      `y` is an integer between `−2^63` and `2^64-1`), the result is computed
      exactly.

      Otherwise, both operands `x` and `y` are converted to the nearest 64 bits
      float (excluding `NaN` and infinity), and we compute the result as a 64
      bits float. This result is then converted back to a rational. In this
      case, **be aware that both the conversion from rationals to floats, and
      the power operation, might incur rounding errors**.
      "%
      = fun x n => %pow% x n,
  },

  record = {
    map
      : forall a b. (String -> a -> b) -> { _ : a } -> { _ : b }
      | doc m%"
      Maps a function on every field of a record. The string argument of the function argument is the name of the
      field.

      For example:
      ```nickel
        map (fun s x => s) { hi = 2 } =>
          { hi = "hi" }
        map (fun s x => x + 1) { hello = 1, world = 2 } =>
          { hello = 2, world = 3 }
      ```
      "%
      = fun f r => %record_map% r f,

    fields
      : forall a. { _ : a } -> Array String
      | doc m%"
      Given a record, results in a array of the string representation of all fields in the record.

      ```nickel
        fields { one = 1, two = 2 } =>
          [ "one", "two" ]
      ```
      "%
      = fun r => %fields% r,

    values
      : forall a. { _ : a } -> Array a
      | doc m%"
      Given a record, results in a array containing all the values in that record.

      ```nickel
        values { one = 1, world = "world" }
          [ 1, "world" ]
      ```
      "%
      = fun r => %values% r,

    has_field
      : forall a. String -> { _ : a } -> Bool
      | doc m%"
      Given the name of a field and a record, checks if the record contains the given field.

      ```nickel
        has_field "hello" { one = 1, two = 2 } =>
          false
        has_field "one" { one = 1, two = 2 } =>
          true
      ```
      "%
      = fun field r => %has_field% field r,

    insert
      : forall a. String -> a -> { _ : a } -> { _ : a }
      | doc m%%"
        Insert a new field in a record. `insert` doesn't mutate the original
        record but returns a new one instead.

        ```nickel
        insert "foo" foo { bar = "bar" } =>
          { foo = "foo", bar = "bar }

        {}
        |> insert "file.%{ext}" "data/text"
        |> insert "length" 10*1000 =>
          {"file.txt" = "data/text", "length" = 10000}
        ```
      "%%
      = fun field content r => %record_insert% field r content,

    remove
      : forall a. String -> { _ : a } -> { _ : a }
      | doc m%"
        Remove a field from a record. `remove` doesn't mutate the original
        record but returns a new one instead.

        ```nickel
        remove "foo" foo { foo = "foo", bar = "bar" } =>
          { bar = "bar }
        ```
      "%
      = fun field r => %record_remove% field r,

    update
      : forall a. String -> a -> { _ : a } -> { _ : a }
      | doc m%"
        Update a field of a record with a new value. `update` doesn't mutate the
        original record but returns a new one instead. If the field to update is absent
        from the given record, `update` simply adds it.

        ```nickel
        remove "foo" foo { foo = "foo", bar = "bar" } =>
          { bar = "bar" }
        ```

        As opposed to overriding a value with the merge operator `&`, `update`
        will only change the specified field and won't automatically update the other
        fields which depend on it:

        ```nickel
        { foo = bar + 1, bar | default = 0 } & { bar = 1 } =>
          { foo = 2, bar = 1 }
        update "bar" 1 {foo = bar + 1, bar | default = 0 } =>
          { foo = 1, bar = 1 }
        ```
      "%
      = fun field content r =>
        let r =
          if %has_field% field r then
            %record_remove% field r
          else
            r
        in
        %record_insert% field r content,

    map_values
      : forall a b. (a -> b) -> { _ : a } -> { _ : b }
      | doc m%"
      Maps a function over every field value of a record.

      For example:
      ```nickel
        map_values (fun x => x + 1) { hi = 2 } =>
          { hi = 3 }
        map (fun x => x + 1) { hello = 1, world = 2 } =>
          { hello = 2, world = 3 }
      ```
    "%
      = fun f => map (fun _field value => f value),

    to_array
      : forall a. { _ : a } -> Array { field : String, value : a }
      | doc m%"
      Converts a record to an array of key-value pairs.

      For example:
      ```nickel
        to_array { hello = "world", foo = "bar" } =>
          [ { field = "hello", value = "world" },
            { field = "foo", value = "bar" }
          ]
      ```
    "%
      = fun record =>
        record
        |> fields
        |> array.map (fun field' => { field = field', value = record."%{field'}" }),

    from_array
      : forall a. Array { field : String, value : a } -> { _ : a }
      | doc m%"
      Convert an array of key-value pairs into a record.

      For example:
      ```nickel
        from_array [ { field = "hello", value = "world" },
                     { field = "foo", value = "bar" }
                   ]
          => { hello = "world", foo = "bar" }
      ```
    "%
      = fun bindings =>
        bindings
        |> array.map (fun binding => { "%{binding.field}" = binding.value })
        |> merge_all,

    is_empty
      : forall a. { _ : a } -> Bool
      | doc m%"
      Check whether a record is empty.

      ```nickel
      is_empty {} => true
      is_empty { foo = 1 } => false
      ```
    "%
      = (==) {},

    merge_all
      | forall a. Array { _ : a } -> { _ : a }
      | doc m%"
      Merge an array of records.

      For example:
      ```nickel
      merge_array [ { foo = 1 }, { bar = 2 } ]
        => { foo = 1, bar = 2 }
      ```
    "%
      = array.fold_left (&) {},

    filter
      : forall a. (String -> a -> Bool) -> { _ : a } -> { _ : a }
      | doc m%"
      Filter a record using the given function, which is passed the name and
      value for each key to make a decision.

      For example:
      ```nickel
      filter (fun name x => x % 2 == 0) { even = 2, odd = 3 }
        => { even = 2 }
      ```
    "%
      = fun f record =>
        record
        |> to_array
        |> array.filter (fun { field, value } => f field value)
        |> from_array,
  },

  string = {
    BoolLiteral
      | doc m%"
      Contract to enforce the value is a string that represents a boolean literal. Additionally casts "True" to "true"
      and "False" to "false".

      For example:
      ```nickel
        ("True" | BoolLiteral) =>
          "true"
        ("hello" | BoolLiteral) =>
          error
        (true | BoolLiteral) =>
          error
      ```
      "%
      = fun l s =>
        if %typeof% s == `String then
          if s == "true" || s == "True" then
            "true"
          else
            if s == "false" || s == "False" then
              "false"
            else
              %blame% (%label_with_message% "expected \"true\" or \"false\", got %{s}" l)
        else
          %blame% (%label_with_message% "not a string" l),

    NumLiteral
      | doc m%"
      Contract to enforce the value is a string that represends a numerical value.

      For example:
      ```nickel
        ("+1.2" | NumLiteral) =>
          "+1.2"
        ("5" | NumLiteral) =>
          "5"
        (42 | NumLiteral) =>
          error
      ```
      "%
      = let pattern = m%"^[+-]?(\d+(\.\d*)?(e[+-]?\d+)?|\.\d+(e[+-]?\d+)?)$"% in
      let is_num_literal = %str_is_match% pattern in
      fun l s =>
        if %typeof% s == `String then
          if is_num_literal s then
            s
          else
            %blame% (%label_with_message% "invalid num literal" l)
        else
          %blame% (%label_with_message% "not a string" l),

    Character
      | doc m%"
      Contract to enforce the value is a character (i.e. a string of length 1).

      For example:
      ```nickel
        ("e" | Character) =>
          "e"
        ("#" | Character) =>
          "#"
        ("" | Character) =>
          error
        (1 | Character) =>
          error
      ```
      "%
      = fun l s =>
        if %typeof% s == `String then
          if length s == 1 then
            s
          else
            %blame% (%label_with_message% "length different than one" l)
        else
          %blame% (%label_with_message% "not a string" l),

    Stringable
      | doc m%"
      Contract to enforce the value is convertible to a string via
      `builtin.to_string` or `string.from`. Accepted values are:

      - numbers
      - booleans
      - strings
      - enum tags
      - null

      For example:
      ```nickel
        (`Foo | Stringable) =>
          `Foo
        (false | Stringable) =>
          false
        ("bar" ++ "foo" | Stringable) =>
          "barfoo"
        ({foo = "baz"} | Stringable) =>
          error
      ```
      "%
      = contract.from_predicate (fun value =>
        let type = builtin.typeof value in
        value == null
        || type == `Number
        || type == `Bool
        || type == `String
        || type == `Enum),

    NonEmpty
      | doc m%"
      Contract to enforce the value is a non-empty string.

      For example:
      ```nickel
        ("" | NonEmpty) =>
          error
        ("hi!" | NonEmpty) =>
          "hi!"
        (42 | NonEmpty) =>
          error
      ```
      "%
      = fun l s =>
        if %typeof% s == `String then
          if %str_length% s > 0 then
            s
          else
            %blame% (%label_with_message% "empty string" l)
        else
          %blame% (%label_with_message% "not a string" l),

    join
      : String -> Array String -> String
      | doc m%"
      Joins a array of strings given a separator.

      For example:
      ```nickel
        join ", " [ "Hello", "World!" ] =>
          "Hello, World!"
      ```
      "%
      = fun sep fragments =>
        let length = %length% fragments in
        if length == 0 then
          ""
        else
          let first = %elem_at% fragments 0 in
          let rest =
            %array_slice% 1 length fragments
            |> array.fold_left (fun acc s => acc ++ sep ++ s) ""
          in
          first ++ rest,

    split
      : String -> String -> Array String
      | doc m%"
      Splits a string based on a separator string. The separator string is not included in any string.

      For example:
      ```nickel
      split "," "1,2,3" =>
        [ "1", "2", "3" ]
      split "." "1,2,3" =>
        [ "1,2,3" ]
      ```
      "%
      = fun sep s => %str_split% s sep,

    trim
      : String -> String
      | doc m%"
      Trims whitespace from the start and end of the string.

      For example:
      ```nickel
      trim " hi  " =>
        "hi"
      trim "1   2   3   " =>
        "1   2   3"
      ```
      "%
      = fun s => %str_trim% s,

    characters
      : String -> Array String
      | doc m%"
      Separates a string into its individual characters.

      For example:
      ```nickel
        chars "Hello" =>
          [ "H", "e", "l", "l", "o" ]
      ```
      "%
      = fun s => %str_chars% s,

    codepoint
      | Character -> Number
      | doc m%%"
      Results in the unicode codepoint of the given character if it fits into a single codepoint.

      For example:
      ```nickel
        code "A" =>
          65
        code "%" =>
          37
        code "å" =>
          error
      ```
      "%%
      = fun s => %char_code% s,

    from_codepoint
      | Number -> Character
      | doc m%%"
      Results in the character for a given unicode codepoint.

      For example:
      ```nickel
        from_code 65 =>
          "A"
        from_code 37 =>
          "%"
        from_code 128 =>
          error
      ```
      "%%
      = fun s => %char_from_code% s,

    uppercase
      : String -> String
      | doc m%"
      Results in the uppercase version of the given character (including non-ascii characters) if it exists, the same
      character if not.

      For example:
      ```nickel
        uppercase "a" =>
          "A"
        uppercase "æ" =>
          "Æ"
        uppercase "." =>
          "."
      ```
      "%
      = fun s => %str_uppercase% s,

    lowercase
      : String -> String
      | doc m%"
      Results in the lowercase version of the given character (including non-ascii characters) if it exists, the same
      character if not.

      For example:
      ```nickel
        lowercase "A" =>
          "a"
        lowercase "Æ" =>
          "æ"
        lowercase "." =>
          "."
      ```
      "%
      = fun s => %str_lowercase% s,

    contains
      : String -> String -> Bool
      | doc m%"
      Checks if the first string is part of the second string.

      For example:
      ```nickel
        contains "cde" "abcdef" =>
          true
        contains "" "abcdef" =>
          true
        contains "ghj" "abcdef" =>
          false
      ```
      "%
      = fun subs s => %str_contains% s subs,

    replace
      : String -> String -> String -> String
      | doc m%"
      `replace sub repl String` replaces every occurrence of `sub` in `String` with `repl`.

      For example:
      ```nickel
        replace "cd" "   " "abcdef" =>
          "ab   ef"
        replace "" "A" "abcdef" =>
          "AaAbAcAdAeAfA"
      ```
      "%
      = fun pattern replace s =>
        %str_replace% s pattern replace,

    replace_regex
      : String -> String -> String -> String
      | doc m%"
      `replace_regex regex repl String` replaces every match of `regex` in `String` with `repl`.

      For example:
      ```nickel
        replace_regex "l+." "j" "Hello!" =>
          "Hej!"
        replace_regex "\\d+" "\"a\" is not" "This 37 is a number." =>
          "This \"a\" is not a number."
      ```
      "%
      = fun pattern replace s =>
        %str_replace_regex% s pattern replace,

    is_match
      : String -> String -> Bool
      | doc m%"
      `is_match regex String` checks if `String` matches `regex`.

      For example:
      ```nickel
        is_match "^\\d+$" "123" =>
          true
        is_match "\\d{4}" "123" =>
          false
      ```

      For example, in the following program, the whole call to
      `string.is_match "[0-9]*\\.?[0-9]+ x"` is re-evaluated at each invocation of
      `is_number`. The regexp will be recompiled 3 times in total:

      ```nickel
      let is_number = fun x => string.is_match "[0-9]*\\.?[0-9]+" x in
      ["0", "42", "0.5"] |> array.all is_number =>
        true
      ```

      On the other hand, in the version below, the partial application of
      `string.is_match "[0-9]*\\.?[0-9]+"` is evaluated once, returning a
      function capturing the compiled regexp. The regexp will only be compiled
      once and for all:

      ```nickel
      let is_number' = string.is_match "[0-9]*\\.?[0-9]+" in
      ["0", "42", "0.5"] |> array.all is_number' =>
        true
      ```
      "%
      = fun regex => %str_is_match% regex,

    find
      : String -> String -> { matched : String, index : Number, groups : Array String }
      | doc m%"
      `find regex String` matches `String` given `regex`. Results in the part of `String` that matched, the index of the
      first character that was part of the match in `String`, and a arrays of all capture groups if any.

      For example:
      ```nickel
        find "^(\\d).*(\\d).*(\\d).*$" "5 apples, 6 pears and 0 grapes" =>
          { matched = "5 apples, 6 pears and 0 grapes", index = 0, groups = [ "5", "6", "0" ] }
        find "3" "01234" =>
          { matched = "3", index = 3, groups = [ ] }
      ```

      Note that this function may perform better by sharing its partial application between multiple calls,
      because in this case the underlying regular expression will only be compiled once (See the documentation
      of `string.is_match` for more details).
      "%
      = fun regex => %str_find% regex,

    length
      : String -> Number
      | doc m%"
      Returns the length of the string, as measured by the number of UTF-8
      [extended grapheme clusters](https://unicode.org/glossary/#extended_grapheme_cluster).

      Generally speaking, this gives the number of "visible" glyphs in the string.

      For example:
      ```nickel
        length "" =>
          0
        length "hi" =>
          2
        length "四字熟語" =>
          4
        length "👨🏾‍❤️‍💋‍👨🏻" =>
          1
      ```
      "%
      = fun s => %str_length% s,

    substring
      : Number -> Number -> String -> String
      | doc m%"
      Takes a slice from the string. Errors if either index is out of range.

      For example:
      ```nickel
        substring 3 5 "abcdef" =>
          "de"
        substring 3 10 "abcdef" =>
          error
        substring (-3) 4 "abcdef" =>
          error
      ```
      "%
      = fun start end s => %str_substr% s start end,

    from
      | Stringable -> String
      | doc m%"
      Converts a correct value to a string representation. Same as
      `builtin.to_string`.

      For example:
      ```nickel
      from 42 =>
        "42"
      from `Foo =>
        "Foo"
      from null =>
        "null"
      "%
      = fun x => %to_str% x,

    from_number
      | Number -> String
      | doc m%"
      Converts a number to its string representation.

      For example:
      ```nickel
      from_number 42 =>
        "42"
      ```
      "%
      = from,

    # from_enum | < | Dyn> -> String = fun tag => %to_str% tag,
    from_enum
      | enum.Tag -> String
      | doc m%"
      Converts an enum variant to its string representation.

      For example:
      ```nickel
      from_enum `MyEnum =>
        "MyEnum"
      ```
      "%
      = from,

    from_bool
      | Bool -> String
      | doc m%"
      Converts a boolean value to its string representation.

      For example:
      ```nickel
        from_bool true =>
          "true"
      ```
      "%
      = from,

    to_number
      | NumLiteral -> Number
      | doc m%"
      Converts a string that represents an integer to that integer.

      For example:
      ```nickel
        to_number "123" =>
          123
      ```
      "%
      = fun s => %num_from_str% s,

    to_bool
      | BoolLiteral -> Bool
      | doc m%"
      Converts a string that represents a boolean to that boolean.

      For example:
      ```nickel
        to_bool "true" =>
          true
        to_bool "True" =>
          true
        to_bool "false" =>
          false
      ```
      "%
      = fun s => s == "true",

    # to_enum | String -> < | Dyn> = fun s => %enum_from_str% s,
    to_enum
      | String -> enum.Tag
      | doc m%"
      Converts any string that represents an enum variant to that enum variant.

      For example:
      ```nickel
        to_enum "Hello" =>
          `Hello
      ```
      "%
      = fun s => %enum_from_str% s,
  }
}
