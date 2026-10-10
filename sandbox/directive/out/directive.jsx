import Note from '@comp/content/Note'
import Warning from '@comp/content/Warning'
import { AsideAB } from '@comp/content/AsideAB'

export default function PendonView() { return (<>
<p>Occaecat Lorem duis irure consequat aliqua commodo dolore duis et voluptate deserunt.
</p>
<h2>Officia Laborum</h2>
<AsideAB type={"asideB"} author={"John Doe"} kind={"kind5"} label={"FooX"} long={"very"} style={"--color: red"} summary={"Lorem officia laborum"}><p>Duis officia <Warning number={24} reason={"aliquip id"} type={"warning"}> pariatur</Warning>- laborum.
</p>
<AsideAB type={"asideA"}><p>Ullamco non enim <Note id={"idNx"} type={"note"}>&#123;`&#125; ullamco<span type="tagX"> consequat</span></Note> velit anim dolor nisi consectetur nisi veniam veniam dolor amet.
</p>
</AsideAB></AsideAB>
</>); }
