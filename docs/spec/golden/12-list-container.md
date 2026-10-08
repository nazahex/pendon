A container decorator above a list decorates the *container*, not the items
(§9.3 L1); the decorator line touches the list it decorates and a typed head
routes through the marker's layer (§9.4):

@@unorderedA{.u, #l03}
- one
- two

The **marker** decides the layer, never the decorator's own type, and an
unclaimed type falls back to the layer default (§9.4, §11 rule 3):

@@{.o}
1. first
2. second

@@unorderedZ{.z}
* star one
* star two
