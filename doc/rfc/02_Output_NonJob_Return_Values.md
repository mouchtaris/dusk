---
direction: Proposal
affects: vm, main/cli
---
# Output non-Job return values from xsim execution

## What happens now

**`make_vm_call2`** (`m/main/src/load_icode.rs:194`) runs `CleanUp(0)` on the return value at `stack[0]` after function execution. `cleanup_value` (`m/vm/src/vm.rs:692-717`) does:
- **Job values** -> calls `Job::cleanup()` -> `into_pipe(false)` -> inherits stdout -> process runs, output goes to terminal. **This works.**
- **Non-Job values** (LitString, Natural, LitBytes, Null, FuncAddr, ArrayView) -> `"No cleanup"` -- silently ignored. **This is the bug.** A function returning a string or number produces no output.

**`run_vm_script`** -- the module wrapper compiles the `m___system_main___` call as the block's final expression (no CleanUp emitted). Inner statements have their own CleanUp/Collect/Pipe. The return value at stack[0] is abandoned. Same bug for the same non-Job values.

## Proposal

### 1. Add `write_retval_to_stdout` in `m/main/src/load_icode.rs`

After CleanUp has run (or after run_vm_script returns), read `stack[0]`. If it's a non-Job value, write it to stdout using `vm.val_as_bytes()` (`m/vm/src/vm.rs:739`). If it's a Job, CleanUp already handled it -- skip.

```rust
pub fn write_retval_to_stdout(vm: &vm::Vm) -> Result<()> {
    let val = te!(vm.stack_get_val(0));
    match val {
        Value::Job(_) => { /* already handled by CleanUp */ }
        Value::Null(_) => { /* nothing to output */ }
        val => {
            let bytes = te!(vm.val_as_bytes(val));
            te!(io::Write::write_all(&mut io::stdout(), bytes));
        }
    }
    Ok(())
}
```

Uses the existing `val_as_bytes` which already handles LitString, LitBytes, DynString (falls through to `val_as_str`), and errors on truly unsupported types.

Natural is not handled by `val_as_bytes`/`val_as_str` -- it errors with "Not a string value". Two options:
- **(a)** Add Natural handling here manually: `Value::Natural(n) => write!(stdout, "{}", n)`
- **(b)** Treat Natural return as no-output (it's typically `0` from the ending-`;` bug anyway)

Leaning **(a)** for correctness.

### 2. Wire into megafront (`m/main/src/cli.rs:505-524`)

After both paths return, call `write_retval_to_stdout(vm)`:

```rust
if let Some(func_addr) = opts.call {
    te!(make_vm_call2(vm, cmp, func_addr, revargs, [...]));
} else {
    te!(run_vm_script(vm, cmp, revargs, (...])));
}
te!(write_retval_to_stdout(vm));
```

**CleanUp(0) stays in `make_vm_call2`** -- no changes to cleanup flow.

### 3. Complete `script_call_getret`'s `todo!()`s (`m/main/src/load_icode.rs:250-271`)

Fill in with proper handling using `val_as_bytes`, and mark impossible states:

- `Null` -> `vec![]`
- `LitString`, `LitBytes`, `DynString` -> `vm.val_as_bytes(val).to_vec()`
- `Natural(n)` -> `format!("{n}").into_bytes()`
- `Job::Null` -> `unreachable!()` (default Job state, never a return value)
- `Job::System` -> `unreachable!()` (only exists after cleanup/collect/pipe instruction; return values are never cleaned)
- `Job::Spec` -> `make_buffer()` + recurse (already implemented)
- `Job::Buffer` -> `take_bytes()` (already implemented)
- `Array`/`ArrayView` -> `temg!("Cannot collect array as output")`
- `FuncAddr`/`SysCallId` -> `temg!("Cannot collect ... as output")`

## Files to modify

1. **`m/main/src/load_icode.rs`** -- add `write_retval_to_stdout`, complete `script_call_getret` todos
2. **`m/main/src/cli.rs`** -- call `write_retval_to_stdout` after dispatch

## Verification

1. `cargo build` + `cargo test`
2. `xsim -c 'def greet = !echo hello; greet;' -l greet` -- Job return, already works via CleanUp
3. `xsim -c 'def val = "literal"; val;' -l val` -- non-Job return, should now print "literal"
4. `echo '"just a string"' | xsim -c -` -- main script non-Job return
