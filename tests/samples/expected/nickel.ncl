# All the the content of this file is extracted from https://github.com/tweag/nickel/tree/master/examples

# This is a comment.
{
  arrays_arrays = let my_array_lib =
    {
      map : forall a b. (a -> b) -> Array a -> Array b = fun f arr =>
        if arr == []
        then
          []
        else
          let head =
            array.head arr
          in
          let tail =
            array.tail arr
          in
          [ f head ] @ map f tail,
      fold : forall a b. (a -> b -> b) -> Array a -> b -> b = fun f arr first =>
        if arr == []
        then
          first
        else
          let head =
            array.head arr
          in
          let tail =
            array.tail arr
          in
          f head (fold f tail first),
    }
  in
  # Compute `7!`
  let l =
    my_array_lib.map ( fun x => x + 1) [ 1, 2, 3, 4, 5, 6 ]
  in
  my_array_lib.fold ( fun x acc => x * acc) l 1,
  fibonacci_fibonacci = let rec fibonacci =
    fun n =>
      if n == 0
      then
        0
      else
        if n == 1
        then
          1
        else
          fibonacci (n - 1) + fibonacci (n - 2)
  in
  fibonacci 10,
  merge_main = let server =
    import "server.ncl"
  in
  let security =
    import "security.ncl"
  in
  server & security & { firewall.enabled = false } ,
  merge_server = {
    server.host.ip = "182.168.1.1",
    server.host.port = 80,
    server.host.name = "hello-world.net",
  } ,
  merge_security = {
    server.host.options = "TLS",
    firewall.enabled | default = true ,
    firewall.type = "iptables",
    firewall.open_ports = [ 21, 80, 443 ],
  } ,
  polymorphism_polymorphism =
  # First projection, statically typed
  let fst : forall a b. a -> b -> a =
    fun x y => x
  in
  # Evaluation function, statically typed
  let ev : forall a b. (a -> b) -> a -> b =
    fun f x => f x
  in
  let id : forall a. a -> a =
    fun x => x
  in
  (ev id (fst 5 10) == 5 : Bool ),
  simple-contracts_simple-contract-bool =
  # Example of simple custom contract, parametrized by a first argument.
  # In practice, for this kind of simple predicate, one should rather use
  # `contract.from_predicate`
  let EqualsTo =
    fun reference_value label value =>
      if reference_value == value
      then
        value
      else
        contract.blame label
  in
  let AlwaysTrue =
    EqualsTo true
  in
  let AlwaysFalse =
    EqualsTo false
  in

  # This contract says: `not` requires its argument to be true, and in return
  # promise that the return value is false.
  # Try passing `false` to `not`, or to use the identity function (replacing `!x`
  # by `x`) to see contract errors appear.
  let not | AlwaysTrue -> AlwaysFalse =
    fun x => ! x
  in
  not true ,
  simple-contracts_simple-contract-div =
  # /!\ THIS EXAMPLE IS EXPECTED TO FAIL
  # Illustrates a basic contract violation.
  let Even =
    fun label value =>
      if builtin.is_num value && value % 2 == 0
      then
        value
      else
        contract.blame label
  in
  let DivBy3 =
    fun label value =>
      if builtin.is_num value && value % 3 == 0
      then
        value
      else
        contract.blame label
  in
  # Will cause an error! 4 is not divisible by 3.
  (4 | Even | DivBy3)
}
