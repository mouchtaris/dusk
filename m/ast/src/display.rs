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
