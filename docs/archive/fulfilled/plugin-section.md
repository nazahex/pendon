# plugin-sectionize Rework

- We need to completely rework `plugin-sectionize`.
- Currently, `plugin-sectionize` isn't working correctly. It should wrap each heading according to its depth, using an ID derived from the heading's ID or slug. (Check the original file to confirm; I'm not entirely sure myself.)
- Rename it to `plugin-section` and `section`.
- We need to support `@@type` and `{...}` syntax, similar to other plugins. My recommendation is to place them above the heading.
- Currently, there is a decorato r above the heading, even though the heading already has its own extras attached to the `#`. To avoid redundancy, the decorator above should be repurposed for the section.
- We need specific syntax to raise the section level without waiting for a higher-level heading, and to lower the section level without needing a lower-level heading.
  - Recommendation: `<--->` to go up, `>---<` to go down.
- Configuration is needed in `task.section.custom`.

## Priority

Example:

```md
@@sectionA{`slug-section`, #sectionID}
###[slug-head]("Heading X")@@headingX{`slug-head-extras`, #headingID} Heading Title
```

The section ID is derived based on this order of priority: #sectionID > `slug-section` > `slug-head` > `slug-head-extras` > #headingID > heading-title.
Meanwhile, `slug-section` is derived from: `slug-section` > `slug-head` > `slug-head-extras` > #headingID > heading-title.

If `plugin-section` set, heading cannot own `id`, it is always transfered to section.
