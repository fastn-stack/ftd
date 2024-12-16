# EBNF 

```chatinput
<fastn> ::= <comment>* <module_comment>? <item>*
<item> ::=  <comment> | <section>


<module_comment> ::= (<single_module_comment> <new_line>)* <single_module_comment> (<new_line> | <eof>)
<single_module_comment> ::= <doc_comment_marker> <opt_whitespace> <sentence>? <opt_whitespace> 

<comment> ::= (<single_comment> <new_line>)* <single_comment> (<new_line> | <eof>)
<single_comment> ::= <comment_marker> <opt_whitespace> <sentence>? <opt_whitespace>


<section> ::= <start_marker> <opt_whitespace> <section_declaration>? <new_line>?
<section_declaration> ::= (<section_kind> <whitespace>)? <section_name> <colon> <opt_whitespace> <section_value>? <opt_whitespace>
<section_kind> ::= <word>
<section_name> ::= <word>
<section_value> ::= <word>



<start_marker> ::= "--"
<end_marker> ::= "--" <whitespace> "end:" <opt_whitespace>
<new_line> ::= "\n"
<eof> ::= "\n"
<colon> ::= ":"
<comment_marker> ::= ";;"
<doc_comment_marker> ::= ";;;"

<sentence> ::= <word> <opt_whitespace> | <word>
<opt_whitespace> ::= " "*
<whitespace> ::= " "+
<word> ::= <character>+
<character> ::= <letter> | <digit>
<letter> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
```