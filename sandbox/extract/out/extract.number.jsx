import { Cite } from "@comp/Cite";

export const headings = [{ id: "heading-satu", level: 2, text: "Heading Satu" }];
export const references = {
  doe2022: {
    authors: [{ family: "Doe", given: "Jane" }],
    issuedDate: { year: 2022 },
    title: "Extracting Structured Data",
    type: "article",
  },
  smith2020: {
    authors: [{ family: "Smith", given: "John" }],
    issuedDate: { year: 2020 },
    title: "A Book About Extraction",
    type: "book",
  },
};
export const citations = [
  {
    citation_number: "1",
    depth: 1,
    instance_index: "1",
    loc: "hlm. 12",
    name: "sup",
    ref_id: "smith2020",
  },
  {
    citation_number: "2",
    depth: 1,
    instance_index: "2",
    loc: "§3",
    name: "sup",
    ref_id: "doe2022",
  },
];
export const headingsA = [
  {
    depth: 2,
    foo: "bar",
    id: "heading-satu",
    inner_text: "Heading Satu",
    level: "2",
    title: "Title Heading 1",
    type: "HeadingA",
  },
];
export const headingsB = [
  {
    depth: 2,
    foo: "bar",
    id: "heading-satu",
    inner_text: "Heading Satu",
    level: "2",
    title: "Title Heading 1",
    type: "HeadingA",
  },
];
export default function PendonView() {
  return (
    <>
      <h2 foo="bar" id="heading-satu" title="Title Heading 1" type="HeadingA">
        Heading Satu
      </h2>
      <div type="MarkerX">akan bernomor 1</div>
      <div type="MarkerX">akan bernomor 1</div>
      <div type="MarkerY">akan bernomor 2</div>
      <div type="MarkerY">akan bernomor 2.1</div>
      <div type="MarkerZ">
        akan bernomor 2.2 &#123;&#123;MarkerZ&#125;&#125;%%%%numberA akan bernomor 2.2.1
        &#123;&#123;MarkerZ&#125;&#125;%%%%%%numberA akan bernomor 2.2.1.1.1
        &#123;&#123;MarkerC&#125;&#125;%%numberB akan bernomor 2
        &#123;&#123;MarkerZ&#125;&#125;%%numberB akan bernomor 3
        &#123;&#123;MarkerA&#125;&#125;%%%%%%numberB akan bernomor 3.1.1.1.1
      </div>
      <pre lang="html">
        <code>&lt;p&gt;akan bernomor 4&lt;/p&gt;</code>
      </pre>
      <p>
        <img alt="image foo" src="/foo.png" />
        @@ImageA&#123;foo:&quot;bar&quot;&#125;%%numberA akan bernomor 3
      </p>
      <p>
        Kutipan juga diekstrak dari berkas referensi eksternal:{" "}
        <Cite
          reference={references["smith2020"]}
          citation_number={1}
          instance_index={1}
          loc="hlm. 12"
        />{" "}
        dan{" "}
        <Cite reference={references["doe2022"]} citation_number={2} instance_index={2} loc="§3" />.
      </p>
    </>
  );
}
