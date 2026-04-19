---
direction: Proposal
affects: main/load_icode, main/cli
---
# Output non-Job return values from xsim

`CleanUp` skips non-Job values with "No cleanup". After CleanUp, extract `stack[0]` and write it out.

## Change

Add to `m/main/src/load_icode.rs`:

```rust
pub fn write_val(vm: &mut vm::Vm, val: &vm::Value, dest: &mut impl io::Write) -> Result<()> {
    // follows inject_val (m/vm/src/syscall.rs:44)
    match val {
        &Value::LitString(value::LitString(id)) => te!(dest.write_all(te!(vm.get_string_id(id)).as_bytes())),
        &Value::DynString(value::DynString(id)) => te!(dest.write_all(te!(vm.get_dynstring_id(id)).as_bytes())),
        &Value::LitBytes(value::LitBytes(id)) => te!(dest.write_all(te!(vm.get_bytes_id(id)))),
        Value::Natural(n) => te!(write!(dest, "{}", n)),
        &Value::Job(value::Job(id)) => {
            te!(te!(vm.get_job_mut(id)).make_buffer());
            let job::Job::Buffer(buf) = te!(vm.get_job_mut(id)) else { unreachable!() };
            te!(dest.write_all(buf.as_bytes()));
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                let val = te!(vm.stack_get_val(ptr + i)).to_owned();
                te!(write_val(vm, &val, dest));
            }
        }
        Value::ArrayView(view) => {
            te!(view.forall(vm, |vm, val| write_val(vm, val, dest)));
        }
        &Value::FuncAddr(value::FuncAddr(addr)) => te!(write!(dest, "<func@{}>", addr)),
        &Value::SysCallId(value::SysCallId(id)) => te!(write!(dest, "<syscall@{}>", id)),
        Value::Null(_) => {}
    }
    Ok(())
}

pub fn get_retval(vm: &mut vm::Vm) -> Result<Vec<u8>> {
    let val = te!(vm.stack_get_val(0)).to_owned();
    match val {
        Value::Job(_) | Value::Null(_) => Ok(vec![]),
        val => {
            let mut buf = Vec::new();
            te!(write_val(vm, &val, &mut buf));
            Ok(buf)
        }
    }
}
```

Wire in `m/main/src/cli.rs` after the run/call dispatch:

```rust
let retval = te!(get_retval(vm));
if !retval.is_empty() {
    te!(io::Write::write_all(&mut io::stdout(), &retval));
}
```

CleanUp(0) stays in `make_vm_call2`. No changes to cleanup flow.

Also: complete `script_call_getret` todos using `write_val`. Mark `Job::Null` and `Job::System` as `unreachable!()`.

## Expected behavior

```
(xsim -c "def val = 12;"            -l val  ) = 12
(xsim -c "def greet = !echo hello;" -l greet) = hello
(xsim -c '"just a string"'                  ) = just a string
(xsim -c "def val = 12; val"                ) = 12
(xsim -c "def val = 12; val;"               ) = ""   (ending-; bug)
```
