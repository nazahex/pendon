import { Cite } from '@comp/Cite'

export const references = {"alpha":{"authors":[{"family":"A","given":"A"}],"issuedDate":{"year":2020},"title":"Alpha","type":"book"},"beta":{"authors":[{"family":"B","given":"B"}],"issuedDate":{"year":2021},"title":"Beta","type":"article"},"gamma":{"authors":[{"family":"G","given":"G"}],"issuedDate":{"year":2022},"title":"Gamma","type":"book"}};
export const citations = [{"citation_number":1,"reference_id":"alpha"},{"citation_number":2,"reference_id":"beta"},{"citation_number":3,"reference_id":"gamma"}];
export const citationsC = [{"citation_number":"3"}];
export default function PendonView() { return (<>
<p>This fixture is P.1.a/b/c of <code>docs/issues/plugin-extract.md</code>: three <code>cite</code> blocks over
one element. Two name <code>CiteA</code>/<code>CiteB</code> and write two destinations each in two formats
through one block; the third names no type — <code>_</code>, the safety net for the types no
block names — and takes <code>citationsC</code> away from the block before it, because a later
<code>export</code> of the same name wins entirely (D-6). <code>references</code> still lists every entry:
extraction reads the document, it does not cite anything.
</p>
<p>A <Cite citation_number={1} reference_id="alpha" type="CiteA" />, B <Cite citation_number={2} reference_id="beta" type="CiteB" />, C <Cite citation_number={3} reference_id="gamma" />.
</p>

</>); }
