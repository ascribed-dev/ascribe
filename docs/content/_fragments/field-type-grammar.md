```abnf
field-type      = field-base [ "?" ]
field-base      = scalar / enum / list-type / "object"    ; "object" in table form only
scalar          = "string" / "number" / "boolean" / "date"
list-type       = "list" "(" OWS ( scalar / enum / "object" ) OWS ")"
                                        ; "list(object)" in table form only

attribute-type  = attribute-base [ "?" ]
attribute-base  = "string" / "number" / "boolean" / enum / set-type
set-type        = "set" "(" OWS ( "string" / enum ) OWS ")"

enum            = "enum" [ "(" OWS enum-value *( OWS "," OWS enum-value ) OWS ")" ]
                                        ; bare "enum" in table form only, with "values"
enum-value      = 1*( ALPHA / DIGIT / "-" / "_" / "." )
```

Spaces are allowed only where `OWS` appears, and nowhere else. The canonical spelling, which `ascribe fmt` writes, has no spaces except one after each comma: `enum(a, b)`.
