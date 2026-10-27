import { DireA } from '@comp/DireA'

export const direA = [{"depth":1,"inner_text":"ipsum","name":"span","type":"DirA"},{"depth":1,"inner_text":"ipsum dolor","name":"span","type":"DirA"}];
export default function PendonView() { return (<>
<p>Lorem Ipsum
</p>
<div slug="foo-a" title="Foo B" type="DirecA"><p>Lorem DirecA <DireA type={"DirA"}> ipsum</DireA>
</p>
<div type="DirecB"><p>Lorem DirecB <DireA type={"DirA"}> ipsum <span type="DirC"> dolor</span></DireA>.
</p>
</div><p>Lorem DirecA 2
</p>
</div><p>Lorem <span type="ipsum"></span> dolor.
</p>
<div type="foo"></div>
</>); }
