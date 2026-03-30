" Vim syntax file for the Dust scripting language
" Language: Dust
" Maintainer: dusk
" Latest Revision: 2026-03-26

if exists("b:current_syntax")
  finish
endif

" ── Comments ─────────────────────────────────────────────────────────
syn match dustComment "#.*$" contains=dustTodo
syn keyword dustTodo TODO FIXME XXX NOTE contained

" ── Keywords ─────────────────────────────────────────────────────────
syn keyword dustKeyword def let val src
syn keyword dustKeyword include include_str
syn keyword dustKeyword for_each in
syn keyword dustKeyword new if

" ── Binding definitions (highlight the name after keyword) ───────────
syn match dustDefName /\(def\|let\|val\|src\)\s\+\zs[a-zA-Z_][a-zA-Z0-9:.,_=/$-]*/

" ── System commands (!name) ──────────────────────────────────────────
syn match dustSystemCmd /![a-zA-Z_][a-zA-Z0-9:.,_=/-]*/

" ── Variables ($name) ────────────────────────────────────────────────
syn match dustVariable /\$[a-zA-Z_][a-zA-Z0-9:.,_=/-]*/

" ── Slices ($name[...]) ──────────────────────────────────────────────
syn match dustSlice /\$[a-zA-Z_][a-zA-Z0-9:.,_=/-]*\[/me=e-1

" ── Dereference (*name) ──────────────────────────────────────────────
syn match dustDeref /\*[a-zA-Z_][a-zA-Z0-9:.,_=/-]*/

" ── Strings ──────────────────────────────────────────────────────────
syn region dustString start=/"/ skip=/\\"/ end=/"/ contains=dustEscape
syn region dustString start=/'/ end=/'/
" Order matters: longer hash counts must come first
syn region dustRawString start=/r####"/ end=/"####/
syn region dustRawString start=/r###"/ end=/"###/
syn region dustRawString start=/r##"/ end=/"##/
syn region dustRawString start=/r#"/ end=/"#/
syn region dustRawString start=/r"/ end=/"/
syn match dustEscape /\\./ contained

" ── Numbers ──────────────────────────────────────────────────────────
syn match dustNumber /\<[0-9]\+\>/

" ── Paths ────────────────────────────────────────────────────────────
syn match dustPath /\.\.\?\/[a-zA-Z0-9:.,_=/-]*/
syn match dustPath /\~\/[a-zA-Z0-9:.,_=/-]*/
" Absolute paths (after whitespace or operator to avoid false matches)
syn match dustAbsPath /\(^\|\s\|[;={(<>]\)\zs\/[a-zA-Z0-9:.,_=/-]\+/

" ── Options ──────────────────────────────────────────────────────────
syn match dustLongOpt /--[a-zA-Z0-9:.,_=/-]\+/
syn match dustShortOpt /\s\zs-[a-zA-Z][a-zA-Z0-9:.,_=/-]*/

" ── Operators ────────────────────────────────────────────────────────
syn match dustOperator /[=<>@]/
syn match dustDelimiter /[;,]/

" ── Environment variable assignment (NAME = value) ───────────────────
syn match dustEnvVar /[A-Z][A-Z0-9_]*\ze\s*=/

" ── Highlight links ──────────────────────────────────────────────────
hi def link dustComment     Comment
hi def link dustTodo        Todo
hi def link dustKeyword     Keyword
hi def link dustDefName     Function
hi def link dustSystemCmd   Special
hi def link dustVariable    Identifier
hi def link dustSlice       Identifier
hi def link dustDeref       PreProc
hi def link dustString      String
hi def link dustRawString   String
hi def link dustEscape      SpecialChar
hi def link dustNumber      Number
hi def link dustPath        String
hi def link dustAbsPath     String
hi def link dustLongOpt     Type
hi def link dustShortOpt    Type
hi def link dustOperator    Operator
hi def link dustDelimiter   Delimiter
hi def link dustEnvVar      Constant

let b:current_syntax = "dust"
