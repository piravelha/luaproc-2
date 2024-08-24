
if exists("b:current_syntax")
  finish
endif

syn match luaType "\<[A-Z][a-zA-Z_0-9]*\>"
syn keyword luaKeyword if then else end function do while for repeat until return local true false nil in or and not
syn match luaIdentifier "\<[a-z_][a-zA-Z_0-9]*\>"

syn match luaFunction "\<[a-z_][a-zA-Z_0-9]*\s*\((\|{\|\"\|\[\[\)\@="

syn keyword luaStatement goto
syn match luaStatement "#\(define\|undef\|end\|ifdef\|ifndef\|endif\|include\|else\)\>"

syn match luaKeyword "::"
syn match luaSpecial "#\([a-zA-Z_][a-zA-Z_0-9]*#\)\@="
syn match luaSpecial "\(#[a-zA-Z_][a-zA-Z_0-9]*\)\@<=#"
syn match luaSpecial "#\?\.\.\."
syn match luaSpecial "##"
syn match luaMacro "\<[a-zA-Z_]\w*!"

syn match luaNumber "-\?\d\+\(\.\d\+\)\?"
syn match luaString "\"\([^\"\\]\|\\.\)*\""
syn match luaString "'\([^'\\]\|\\.\)*'"
syn region luaString start=+\[\[+ end=+\]\]+

syn match luaComment "--.*$"

highlight link luaSpecial jsRegexpString
highlight link luaKeyword StorageClass
highlight link luaMacro Constant
highlight link luaStatement Statement
highlight link luaNumber Number
highlight link luaString String
highlight link luaFunction Function
highlight link luaComment Comment
highlight link luaIdentifier Identifier
highlight link luaType Type
