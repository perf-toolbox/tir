# The Language Parsing Library

Language Parsing Library is TIR's built-in parser combinator library, similar to
`nom`, `winnow` or `chumsky`. It is aimed at providin flexible yet performant
interfaces for any built-in DLS within TIR domain.

## Rationale

One may wonder why not just use any of the existing libraries. There're a few
reasons to take this road:

1. Existing libraries are often too abstract and too hard to use. With too many
   customization points, they often become a burden when supporting multiple
   built-in DSLs. This includes things like error handling that is often lacking
   features required for mature language tooling. Rather then being a one stop
   shop for every parsing problem, we focus on things that matter for us:
   unified error handling and diagnostic engine, source tracking on by default,
   common sub-parsers.
2. Lack of features. Surprisingly, generic parser combinators usually lack
   things that are actually needed for a real-world compiler or DSL, like
   comment parsing or usable support for separate lexing and parsing stages.
3. Performance. Production compilers deal with huge amounts of data, and the
   hotspots can be in the most unexpected places. To have better control over
   our compile times, we need to make sure every part of the pipeline can be
   easily changed.
