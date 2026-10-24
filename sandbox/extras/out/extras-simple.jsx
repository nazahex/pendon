import DirecA from '@comp/content/DirecA'

export const frontmatter = {"fm1":"This is a frontmatter label"};
export default function PendonView() { return (<>
<DirecA bags={[{fruits: ["banana", "apple"]}, {vegs: "carrot"}]} customers={["Budi Ali", "Siska Ratna"]} labels={frontmatter.fm1} token={{content: {original: "Foo bar bur.", text: "Lorem Ipsum"}, def: {theme: {accent: "#0af", scale: [1, 2]}}}} type="DirecA" />
</>); }
