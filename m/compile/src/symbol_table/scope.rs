use super::*;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    sym_info: Info,
    sym_indx: Indx,
}
buf::sd_struct![Scope, sym_info, sym_indx];

type Info = Vec<(String, SymID)>;
type Indx = Map<String, usize>;

#[derive(Default, Clone, Debug)]
pub struct SymID {
    sym_id: usize,
    sym_info: SymInfo,
}
buf::sd_struct![SymID, sym_id, sym_info];

impl<S: Ref<Scope> + ?Sized> ApiRef for S {}
impl<S: Mut<Scope> + ?Sized> ApiMut for S {}

pub(crate) type SymDetails<'a> = (&'a str, &'a mut SymID, Option<usize>);

pub(crate) trait ApiMut: Mut<Scope> {
    /// Return
    /// - `&str` name
    /// - `&mut SymID`: (sym_id * sym_info)
    /// - `usize` prev sym_id (as usize because can't borrow all together)
    fn insert(&mut self, name: String, sym_info: SymInfo) -> SymDetails {
        // Symbol ID: next in line in scope
        let sym_id: usize = as_ref(self).next_id();

        // Entry = (name * info)
        let entry = (name.clone(), SymID { sym_id, sym_info });

        // Mutate
        let (sym_infos, sym_indx, ..) = parts_mut(self);

        // Insert to index: [name] -> sym_id
        let prev_id = sym_indx.insert(name, sym_id);

        // Insert to store: [sym_id] -> (name * info)
        sym_infos.push(entry);

        // Re-retrieve to return
        let (name, id_ref) = sym_infos.last_mut().expect("just entered item");
        let name = name.as_str();

        // Return
        (name, id_ref, prev_id)
    }
}
pub(crate) trait ApiRef: Ref<Scope> {
    /// Lookup a symbol textual name by SymID
    fn sym_name(&self, sym_id: &SymID) -> Option<&str> {
        let (infos, ..) = parts(self);
        let &SymID { sym_id, .. } = sym_id;

        infos.get(sym_id).map(|(name, _)| name.as_str())
    }

    /// Lookup a symbol textual name by SymInfo.
    ///
    /// This is slower and less reliable that by-SymID, as it has
    /// to traverse all symbols and compare that
    /// `their sym_info == given sym_info`.
    ///
    /// Depending on the assignment of [SymInfo] this can be reliable,
    /// if they are all completely unique. In the current implementation
    /// they probably are.
    ///
    /// Also, the need for this is a remanent from when [SymID] was not
    /// there, which assigns unique ids to symbol when inserted. After
    /// some refactoring in client code, this should go.
    #[deprecated(note = "use sym_name")]
    fn sym_name_from_info(&self, info: &SymInfo) -> Option<&str> {
        let (infos, ..) = parts(self);

        infos.iter().find_map(|(n, x)| {
            if x.sym_info() == info {
                Some(n.as_str())
            } else {
                None
            }
        })
    }

    /// All symbols within the scope, in reverse order of insertion
    fn all_symbols_in_reverse(&self) -> impl Seq<Item = (&str, &SymID)> {
        let (infos, ..) = parts(self);
        infos.iter().rev().map(|(k, v)| (k.as_str(), v))
    }

    fn next_id(&self) -> usize {
        let (infos, ..) = parts(self);
        infos.len()
    }

    /// ## Basic Lookup by name
    ///
    /// This is the most expected lookup in the scope:
    /// exact name match.
    ///
    /// The benefit of this over bluntly looking through
    /// [ApiRef::all_symbols_in_reverse] every time, is that this is probably
    /// optimized by a hash-index.
    ///
    fn lookup_by_name(&self, name: impl Ref<str>) -> Option<&SymID> {
        let (infos, index, ..) = parts(self);
        index.get(name.borrow()).map(|&id| &infos[id].1)
    }
}

fn as_ref(this: &mut (impl Mut<Scope> + ?Sized)) -> &impl ApiRef {
    this.borrow_mut()
}

fn parts_mut(this: &mut (impl Mut<Scope> + ?Sized)) -> (&mut Info, &mut Indx) {
    let Scope {
        sym_info, sym_indx, ..
    } = this.borrow_mut();
    (sym_info, sym_indx)
}
fn parts(this: &(impl Ref<Scope> + ?Sized)) -> (&Info, &Indx) {
    let Scope {
        sym_info, sym_indx, ..
    } = this.borrow();
    (sym_info, sym_indx)
}

impl SymID {
    pub fn sym_info(&self) -> &SymInfo {
        let Self { sym_info, .. } = self;
        sym_info
    }
    pub fn sym_info_mut(&mut self) -> &mut SymInfo {
        let Self { sym_info, .. } = self;
        sym_info
    }
}
