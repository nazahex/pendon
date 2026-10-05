A figure: the extras attach to the outermost node, `w`/`h` stay on the inner
`<img>` and the trailing text is the caption (§7.1).

~?!!h300w800[Alt text](https://res.cloudinary.com/x/upload/fig.webp)@@figureX{.wide, #fig-1, --rotate: "5deg", isLazy} A figure caption with **markup**.

An image node (`~?!`) uses the `img` layer alone:

~?!w600[Alt text](https://res.cloudinary.com/x/upload/img.webp)@@imageX{.thumb, foo: "bar"}

A lazy image inline attaches its extras to the `<img>` element:

Ad ex tempor !?~w300[Alt text](https://res.cloudinary.com/x/upload/lazy.webp)@@imageX{con: "jux", .foo} consectetur.
