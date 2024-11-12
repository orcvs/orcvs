






### Notes




App

  update
    -> source.update

  -> Source

    -> source expressions
    -> parsed expressions
    -> glyphs

  [thread]
  on_interval
    -> execute








### Benchmarks

```
cargo bench  --bench parser_benchmark -- --save-baseline no-inline
```


### Notes to self

    - App.Cursor can be flattened



From Orca

```
  this.isCursor = (x, y) => {
    return x === this.cursor.x && y === this.cursor.y
  }

  this.isMarker = (x, y) => {
    return x % this.grid.w === 0 && y % this.grid.h === 0
  }

  this.isNear = (x, y) => {
    return x > (parseInt(this.cursor.x / this.grid.w) * this.grid.w) - 1 && x <= ((1 + parseInt(this.cursor.x / this.grid.w)) * this.grid.w) && y > (parseInt(this.cursor.y / this.grid.h) * this.grid.h) - 1 && y <= ((1 + parseInt(this.cursor.y / this.grid.h)) * this.grid.h)
  }

  this.isLocals = (x, y) => {
    return this.isNear(x, y) === true && (x % (this.grid.w / 4) === 0 && y % (this.grid.h / 4) === 0) === true
  }

  this.isInvisible = (x, y) => {
    return this.orca.glyphAt(x, y) === '.' && !this.isMarker(x, y) && !this.cursor.selected(x, y) && !this.isLocals(x, y) && !this.ports[this.orca.indexAt(x, y)] && !this.orca.lockAt(x, y)
  }

```


// Split the expression
// [AAA] => [B C]
// Remove current expression from idx
// idx is the middle element of expression [AAA]
// after this operation we are at [A A]
// self.remove_exp(idx);

// We have [A A]
// We are going to map A-1 => B
//                 and A+1 => C
// Any elements of A to the left of A-1 now also B
// Any elements of A to the left of A+1 now also C
// eg
//   [AAAAA]
//   [A AAA]
//   [B CCC]
//
//   [AAAAA]
//   [AAA A]
//   [BBB C]
//
// Create a new expression (C) for A+1 (Right)
// Starts at idx + 1
// Ends is current A end
// {
//     // lft and rgt often refer to the same expression
//     // We cannot have multiple mutable borrows in the same scope
//     // So we split the borrows into separate scopes
//     // Right must be first as we want to capture the end value before modifying the left value
//     let rgt = rgt_exp.borrow();
//     let rgt_end = rgt.end;
//     let exp = Rc::new(RefCell::new(SourceExpression::new(rgt_idx, rgt_end)));
//     self.set_exp_from(rgt_idx, rgt_end, &exp);
// }
// // Update A-1 (Left)
// // Expression now ends at A-1 (Left)
// self.end_exp(lft_idx, &lft_exp);



        /*
            // ... => .I.
            // Create a new expression if the lhs and rhs are None
            //   exp.start = idx
            //   exp.end   = idx

            // .I. => .ID.
            // Append to the lhs expression if Some(lft_exp) and None(rhs)
            //   lhs.end = idx

            // .IDAA. => .ID0A.
            // Replace if Some(lft) and Some(rhs)
            //   // noop

            // .IDAA. => .ID.A.
            // Split if Terminator and Some(lft) and Some(rhs)
            //

            // .ID.A. => .IDAA.
            // Join if None(idx) and Some(lft_exp) and Some(rhs)
            //  lft.end = rgt.end
            //  rgt = lft
            //  self.map[idx] = lft

            // ..DAA. => .IDAA.
            // Prepend if None(lhs) and Some(rhs)
            //  exp.start = idx
        */




### Simplified Design

-> Fixed Array of Chars
   [A, B, C, D]

   On Change
      take while char
        parse
          -> glyphs








[ ][I][1][ ][ ]
[ ][1][ ][ ][ ]
[ ][ ][ ][ ][ ]