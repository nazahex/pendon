§9.5: a decorator line above a heading decorates the **section**, not the
heading — the heading already carries its own extras on the `#` run, so a second
head above it would be redundant:

@@sectionA{`slug-section`}
##[slug-head]("Head title")@@headingX{.lead} Section A

The section id is the first of `#sectionID` > `slug-section` > `[slug]` > extras
slug > the slug of the title; the heading yields its own id to the section.

<--->

A `<--->` marker deepens the outline without waiting for a deeper heading.

>---<

`>---<` closes the innermost section again.

## Bare Section B

Body of section B.
