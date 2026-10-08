A block directive owns its line and its body is parsed as block content (§10.3):

==note
# A heading inside the directive

- the first item
- the second item
==

Nested fences close LIFO, so the innermost directive is the first to close
(§10.3):

==outer
==note
inner body
==
trailing text of the outer
==

A fence with nothing open, and a fence without a type, stay literal text: there
is no anonymous block directive (§10.3, OPEN-B1):

==
==5

An unclosed directive is closed implicitly at the end of the input, with a
warning (§10.3):

==note
this block never closes
