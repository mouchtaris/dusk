use {super::*, std::fmt};

impl<'i> fmt::Display for InvocationTarget<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvocationTarget::InvocationTargetLocal(InvocationTargetLocal((name,))) => {
                write!(f, "{}", name)
            }
            InvocationTarget::InvocationTargetSystemName(InvocationTargetSystemName((name,))) => {
                write!(f, "!{}", name)
            }
            InvocationTarget::InvocationTargetSystemPath(InvocationTargetSystemPath((path,))) => {
                write!(f, "!{}", path)
            }
            InvocationTarget::InvocationTargetDereference(InvocationTargetDereference((var,))) => {
                write!(f, "{var}")
            }
            InvocationTarget::InvocationTargetInvocation(InvocationTargetInvocation((
                invocation,
            ))) => {
                write!(f, "{invocation}")
            }
        }
    }
}

impl<'i> fmt::Display for Path<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Path::AbsPath(AbsPath((path,))) => write!(f, "{}", path),
            Path::RelPath(RelPath((path,))) => write!(f, "{}", path),
            Path::HomePath(HomePath((path,))) => write!(f, "{}", path),
        }
    }
}

impl<'i> AsRef<std::path::Path> for Path<'i> {
    fn as_ref(&self) -> &std::path::Path {
        match self {
            Path::AbsPath(AbsPath((path,)))
            | Path::RelPath(RelPath((path,)))
            | Path::HomePath(HomePath((path,))) => path.as_ref(),
        }
    }
}

impl<'i> fmt::Display for Variable<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self((id,)) = self;
        write!(f, "${id}")
    }
}

impl<'i> fmt::Display for Dereference<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self((id,)) = self;
        write!(f, "*{id}")
    }
}

impl<'i> fmt::Display for Invocation<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self((comment, target, cwd, inp, out, env, args)) = self;
        for line in comment {
            writeln!(f, "{line}")?;
        }
        write!(f, "{target}")?;
        if let Some(cwd) = cwd {
            write!(f, " {cwd}")?;
        }
        for RedirectInput((redir,)) in inp {
            write!(f, " <{redir}")?;
        }
        for RedirectOutput((redir,)) in out {
            write!(f, " >{redir}")?;
        }
        for (name, val) in env {
            write!(f, "i {name}={val}")?;
        }
        for arg in args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}
impl<'i> fmt::Display for InvocationCwd<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InvocationCwd::*;
        match self {
            Path(path) => write!(f, "{path}")?,
            Variable(variable) => write!(f, "{variable}")?,
            BoxInvocation(invocation) => write!(f, "{invocation}")?,
        }
        Ok(())
    }
}
impl<'i> fmt::Display for Redirect<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Redirect::*;
        match self {
            Path(path) => write!(f, "{path}")?,
            Variable(variable) => write!(f, "{variable}")?,
            Dereference(dereference) => write!(f, "{dereference}")?,
            Invocation(invocation) => write!(f, "{invocation}")?,
            Slice(slice) => write!(f, "{slice}")?,
            String(super::String((s,))) => write!(f, "{s}")?,
        }
        Ok(())
    }
}
impl<'i> fmt::Display for InvocationArg<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InvocationArg::*;
        match self {
            Ident(s) => write!(f, "{s}")?,
            Opt(opt) => write!(f, "{opt}")?,
            Path(path) => write!(f, "{path}")?,
            String(super::String((s,))) => write!(f, "{s}")?,
            Variable(variable) => write!(f, "{variable}")?,
            Word(super::Word((s,))) => write!(f, "{s}")?,
            Natural(super::Natural((s,))) => write!(f, "{s}")?,
            Invocation(invocation) => write!(f, "{invocation}")?,
            Slice(slice) => write!(f, "{slice}")?,
        }
        Ok(())
    }
}
impl<'i> fmt::Display for Slice<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self((name, range)) = self;
        write!(f, "{name}{range}")?;
        Ok(())
    }
}
impl<'i> fmt::Display for Opt<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Opt::*;
        match self {
            ShortOpt(super::ShortOpt((s,))) => write!(f, "{s}")?,
            LongOpt(super::LongOpt((s,))) => write!(f, "{s}")?,
        }
        Ok(())
    }
}
impl<'i> fmt::Display for Range<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Range::*;
        match self {
            DoubleRange((a, b)) => write!(f, "[{a}; {b}]")?,
            Index(invocation_arg) => write!(f, "[{invocation_arg}]")?,
        }
        Ok(())
    }
}
