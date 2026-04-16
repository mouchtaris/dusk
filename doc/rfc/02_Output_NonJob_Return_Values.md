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

### 1. Add `get_retval` and `write_retval` in `m/main/src/load_icode.rs`

Split into two parts:
- `get_retval(vm) -> Result<Vec<u8>>` -- extracts the return value from `stack[0]` as bytes
- Caller writes to any `impl io::Write` destination

`get_retval` uses `inject_val` from `m/vm/src/syscall.rs:44` as the model -- it already handles all Value types including Array/ArrayView recursively, and Natural via format buffer. For the return value case:

```rust
pub fn get_retval(vm: &mut vm::Vm) -> Result<Vec<u8>> {
    let val = te!(vm.stack_get_val(0)).to_owned();
    Ok(match val {
        Value::Job(_) => vec![],  // already handled by CleanUp
        Value::Null(_) => vec![],
        _ => {
            let mut buf = Vec::new();
            te!(write_val(vm, &val, &mut buf));
            buf
        }
    })
}
```

`write_val` follows `inject_val` (`m/vm/src/syscall.rs:44`) but writes to `impl io::Write` instead of calling a string callback:

- `LitString` -> `vm.get_string_id(id)` -> write bytes
- `DynString` -> `vm.get_dynstring_id(id)` -> write bytes
- `LitBytes` -> `vm.get_bytes_id(id)` -> write bytes
- `Natural(n)` -> `write!(dest, "{}", n)`
- `Job` -> `job.make_buffer()` -> write buffer bytes
- `Array` -> recurse over elements (same pattern as `inject_val`)
- `ArrayView` -> `view.forall()` recurse (same pattern as `inject_val`)
- `FuncAddr(addr)` -> `write!(dest, "<func@{}>", addr)`
- `SysCallId(id)` -> `write!(dest, "<syscall@{}>", id)`

### 2. Wire into megafront (`m/main/src/cli.rs:505-524`)

After both paths return, get retval and write to stdout:

```rust
if let Some(func_addr) = opts.call {
    te!(make_vm_call2(vm, cmp, func_addr, revargs, [...]));
} else {
    te!(run_vm_script(vm, cmp, revargs, (...])));
}
let retval = te!(get_retval(vm));
if !retval.is_empty() {
    te!(io::Write::write_all(&mut io::stdout(), &retval));
}
```

**CleanUp(0) stays in `make_vm_call2`** -- no changes to cleanup flow.

### 3. Complete `script_call_getret`'s `todo!()`s (`m/main/src/load_icode.rs:250-271`)

Fill in using the same `write_val` mechanism, and mark impossible states:

- `Null` -> `vec![]`
- `LitString`, `LitBytes`, `DynString`, `Natural` -> via `write_val`
- `Array`/`ArrayView` -> via `write_val` (recursive traversal)
- `FuncAddr`/`SysCallId` -> via `write_val`
- `Job::Null` -> `unreachable!()` (default Job state, never a return value)
- `Job::System` -> `unreachable!()` (only exists after cleanup/collect/pipe instruction; return values are never cleaned)
- `Job::Spec` -> `make_buffer()` + recurse (already implemented)
- `Job::Buffer` -> `take_bytes()` (already implemented)

## Files to modify

1. **`m/main/src/load_icode.rs`** -- add `get_retval`, `write_val`; complete `script_call_getret` todos
2. **`m/main/src/cli.rs`** -- call `get_retval` + write to stdout after dispatch

## Expected behavior

```
(xsim -c "def val = 12;"            -l val  ) = 12
(xsim -c "def greet = !echo hello;" -l greet) = hello
(xsim -c '"just a string"'                  ) = just a string

(xsim -c "def val = 12; val;"               ) = ""    (ending-; bug: returns Natural(0))
(xsim -c "def val = 12; val"                ) = 12
(xsim -c "def val = !echo hello; val;"      ) = hello (CleanUp runs on inner statement)
(xsim -c "def val = !echo hello; val"       ) = hello
```
