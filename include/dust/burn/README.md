# burn — perfect forward secrecy, spoken as unix gpg

> `》 ~ 《`
> A key ceremony mirrored on axis-0. Identity signs, ephemera encrypt —
> and the wire only ever sees `》~《`.

`burn` is a tiny Dust toolkit — a **library of primitive words** and a
**narrated CLI** — for doing one round of perfect forward secrecy on top of
ordinary `gpg`. The whole secrecy argument is one sentence:

> the message is encrypted to a **throwaway** key; once that key is burned,
> the ciphertext can never be opened again — not even by you, not even with
> your long-term identity.

Your long-term keys (in `~/.gnupg`) are spent **only** on signatures — they
authenticate the handshake and never touch a ciphertext. Secrecy is carried
entirely by ephemeral keys that live in a per-session ring and are destroyed
at the end. That separation *is* forward secrecy.

## The words

`lib` is ~50 lines. Each word is one movement of *The Burn*:

| word     | ceremony            | gpg it speaks                                             |
|----------|---------------------|----------------------------------------------------------|
| `mint`   | conjure ephemerals  | `--quick-generate-key burn/<h>` in a fresh ring          |
| `cross`  | publics face wire   | `--export` the ephemeral public (`》`)                    |
| `take`   | a public arrives    | `--import` the peer’s ephemeral public into the ring     |
| `sign`   | identity stamps it  | `--local-user <id> --detach-sign` (auth, never secrecy)  |
| `verify` | recognise the seal  | `--verify <sig> <pub>` — this, not encryption, kills MITM |
| `seal`   | convolve → `~`       | `--encrypt --sign` to the peer’s ephemeral                |
| `open`   | condense K, read    | `--decrypt` with your own ephemeral                       |
| `burn`   | the theorem         | `rm -rf` the ring — the ephemeral privates stop existing  |

A **handle** is a party’s name for a session (`alice`, `bob`). Each handle
owns a ring at `~/.burn/<handle>` (a private `GNUPGHOME`) and an ephemeral
userid `burn/<handle>` minted inside it.

## One round, two shells

Alice and Bob already have long-term keys in `~/.gnupg` and know each
other’s identity fingerprint.

```sh
# ── each side mints an ephemeral pair and exports its public ──
burn/lib  mint  alice
burn/lib  cross alice                         > alice.pub     # Alice
burn/lib  sign  alice@host  < alice.pub       > alice.sig     # identity signs it

burn/lib  mint  bob
burn/lib  cross bob                           > bob.pub       # Bob
burn/lib  sign  bob@host    < bob.pub         > bob.sig

# ── cross the wire: swap {*.pub, *.sig}, then verify + take ──
burn/lib  verify bob.sig   bob.pub                            # Alice checks Bob
burn/lib  take   alice     < bob.pub                          # …and keeps his public

burn/lib  verify alice.sig alice.pub                          # Bob checks Alice
burn/lib  take   bob       < alice.pub

# ── seal · send · open ──
echo "meet at the usual place" | burn/lib seal alice bob > c.asc
burn/lib  open  bob         < c.asc                           # Bob reads it

# ── the burn ──
burn/lib  burn  alice
burn/lib  burn  bob          # c.asc is now sealed forever
```

`》a · 《b · c.asc` are all the wire ever held. Steal Alice’s long-term
`《A` a year later and you can forge her *future* — you cannot open the
*past*.

## Running

The narrated ceremony:

```sh
xs2 include/dust/burn/burn
```

The words are ordinary Dust methods; drive them from `xs2` or from your own
`.dust` scripts via `include include/dust/burn/lib;`.

### Status — first go, not yet run here

This is a researched first draft written against the maker’s Dust corpus and
verified idiom-by-idiom against the `spec/` tests (`environment`,
`cwd_from_variable`, `argument_subslicing`, `input_redirection_to_method`).
It has **not** been executed, because the interpreter in this checkout can’t
build here — `m/main` depends on a local path (`/home/bob/code_tools/...`)
that isn’t present. Two things to confirm on a machine that can run `xs2`:

1. a Dust method (`in_ring`) inherits the program’s stdin down into its
   `!sh -c` body — the seal/open/take pipes rely on it;
2. `--quick-generate-key <uid> default default 0` yields an
   **encryption-capable** ephemeral (an encryption subkey) so `seal`’s
   `--recipient` resolves.

Both are standard, but they’re the first things I’d smoke-test.
